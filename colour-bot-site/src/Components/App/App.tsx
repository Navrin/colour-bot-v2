import * as React from 'react';
import * as styles from './styles.module.scss';
import Sidebar from '../Sidebar/Sidebar';
import { observer, inject } from 'mobx-react';
import { UserStore } from '../../stores/User';
import settings from '../../settings';
import { Redirect } from 'react-router';
import { stores } from '../..';
import { observable } from 'mobx';
import { GuildStore } from '../../stores/Guild';
import { RoutingStore } from '../../stores/Routing';
import Notifications from './Notifications';
import { NAVIGATIONS, ContentSection } from './Content';
// import { Link } from 'react-router-dom';

interface IAppProps {
    userStore?: UserStore;
    guildStore?: GuildStore;
    routingStore?: RoutingStore;
}

@inject((allStores: typeof stores) => ({
    userStore: allStores.userStore,
    guildStore: allStores.guildStore,
    routingStore: allStores.routingStore,
}))
@observer
class App extends React.Component<IAppProps> {
    @observable
    currentState?: keyof typeof NAVIGATIONS;

    constructor(props: IAppProps) {
        super(props);

        const [, id] = window.location.pathname.split('/');
        if (id) {
            this.props.userStore!.activeGuild = id;
        }
    }

    public componentDidMount() {
        if (this.props.userStore!.token != null) {
            this.props.userStore!.requestInfo();
            return;
        }

        const code = localStorage.getItem('code');

        if (code == null) {
            localStorage.setItem('last-area', window.location.pathname);

            window.location.href = `${settings.endpoint}/login`;
        } else {
            this.props.userStore!.getToken(code);
        }
    }

    // private validStateForAuth(
    //     props: RouteComponentProps<{ id: string }, StaticContext, any>,
    // ): (
    //     value: [string, { name: string; auth: boolean }],
    //     index: number,
    // ) => any {
    //     return ([, nav]) => {
    //         const valid =
    //             (nav.auth &&
    //                 this.props.userStore!.hasRolePermissions(
    //                     props.match.params.id,
    //                 ) === nav.auth) ||
    //             false;

    //         return valid;
    //     };
    // }

    private navigationRouteRender = (
        p: string,
        id: string,
    ): (() => JSX.Element) => {
        return () => {
            const entity = NAVIGATIONS[p];

            if (entity == null) {
                return <Redirect to="/" />;
            }

            const Render = entity.component;
            return <Render id={id} />;
        };
    };

    public render() {
        const routing = this.props.routingStore!;

        return (
            <div id={styles.App}>
                <Notifications />
                {routing.redirect !== location.pathname &&
                    routing.redirect && <Redirect to={routing.redirect} />}
                <div className={styles.GridItem} id={styles.Header}>
                    SERVERS
                </div>
                <Sidebar />

                {
                    <ContentSection
                        onButtonNavigate={this.buttonNavigate}
                        onNavigationRouteRender={this.navigationRouteRender}
                    />
                }
            </div>
        );
    }

    private buttonNavigate = (k: string) => {
        return () => {
            if (this.props.userStore!.activeGuild == null) {
                location.href = '/';
            }

            this.props.userStore!.activeState = k;
            this.props.routingStore!.redirect = `/${
                this.props.userStore!.activeGuild
            }/${k}`;
        };
    };
}

export default App;
