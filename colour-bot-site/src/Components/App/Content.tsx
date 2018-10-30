import * as React from 'react';
import { observer, inject } from 'mobx-react';
import { UserStore } from '../../stores/User';
import { stores } from '../..';
import { GuildStore } from '../../stores/Guild';
import View from '../Methods/View';
import Create from '../Methods/Create';
import * as styles from './styles.module.scss';
import { NavigationButtons } from './NavigationButtons';
import Roles from '../Methods/Roles';

export const NAVIGATIONS = {
    view: { name: 'View', component: View, auth: false },
    create: { name: 'Create', component: Create, auth: true },
    roles: { name: 'Add Roles', component: Roles, auth: true },
    settings: { name: 'Settings', auth: true },
};

type NavigationRouteRenderCallback = (
    p: string,
    id: string,
) => () => JSX.Element;

const InnerContentRoute = observer(function InnerContentRoute(props: {
    guildStore: GuildStore;
    userStore: UserStore;
    onNavigationRouteRender: NavigationRouteRenderCallback;
}) {
    const state = props.userStore!.activeState;
    if (state == null) {
        return <></>;
    }
    const id = props.userStore!.activeGuild!;
    props.guildStore!.addGuildInfo(id);
    return props.onNavigationRouteRender(state, id)();
});

interface IContentSectionProps {
    onButtonNavigate(k: string): (() => void);
    guildStore?: GuildStore;
    userStore?: UserStore;
    onNavigationRouteRender: NavigationRouteRenderCallback;
}

@inject((allStores: typeof stores) => ({
    guildStore: allStores.guildStore,
    userStore: allStores.userStore,
}))
@observer
export class ContentSection extends React.Component<IContentSectionProps> {
    public render() {
        return (
            <React.Fragment>
                <div className={styles.GridItem} id={styles.Navigation}>
                    {this.props.userStore!.activeGuild ? (
                        <NavigationButtons
                            name={name}
                            onButtonNavigate={this.props.onButtonNavigate}
                        />
                    ) : (
                        'You need to select a guild first.'
                    )}
                </div>
                <div className={styles.GridItem} id={styles.Content}>
                    {
                        <InnerContentRoute
                            guildStore={this.props.guildStore!}
                            userStore={this.props.userStore!}
                            onNavigationRouteRender={
                                this.props.onNavigationRouteRender
                            }
                        />
                    }
                </div>
            </React.Fragment>
        );
    }
}
