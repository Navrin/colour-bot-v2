import * as React from 'react';
import * as styles from './styles.module.scss';
import PersonIcon from '@material-ui/icons/AccountCircle';
import { observer, inject } from 'mobx-react';
import { UserStore, IGuildInfo } from '../../stores/User';
import Scrollbars from 'react-custom-scrollbars';
import { stores } from '../..';
import VerifiedUser from '@material-ui/icons/VerifiedUser';
import { RoutingStore } from '../../stores/Routing';

interface ISidebarProps {
    userStore?: UserStore;
    routingStore?: RoutingStore;
}

export function ScrollArea({ children }: { children: JSX.Element[] }) {
    function vertThumbFunc(): React.StatelessComponent<any> | undefined {
        return props => <div {...props} className="Scrollbar-thumb-themed" />;
    }

    function thumbFunc(): React.StatelessComponent<any> | undefined {
        return props => <div {...props} className="Scrollbar-thumb-themed" />;
    }
    return (
        <Scrollbars
            renderThumbVertical={vertThumbFunc()}
            renderThumbHorizontal={thumbFunc()}
        >
            {children}
        </Scrollbars>
    );
}

function GuildItem(
    props: IGuildInfo & {
        active: boolean;
        onClick: () => void;
        authed: boolean;
    },
) {
    return (
        <div
            onClick={props.onClick}
            aria-selected={props.active}
            className={styles.SidebarItem}
        >
            <div className={styles.SidebarItemText}>
                {props.name} {props.authed && <VerifiedUser />}
            </div>
        </div>
    );
}

@inject((allStores: typeof stores) => ({
    routingStore: allStores.routingStore,
    userStore: allStores.userStore,
}))
@observer
class Sidebar extends React.Component<ISidebarProps> {
    render() {
        const userStore = this.props.userStore!;

        return (
            <div id={styles.Sidebar}>
                {/* {routingStore.redirect && (
                    <Redirect to={routingStore.redirect} />
                )} */}
                <div id={styles.SidebarList}>
                    <ScrollArea>
                        <div id={styles.SidebarListActive}>
                            <h2>Active Guilds</h2>
                            {Array.from(userStore.guilds.values())
                                .filter(i => i.cached)
                                .map(this.makeGuild(userStore, false))}
                        </div>

                        <div id={styles.SidebarListInactive}>
                            <h2>
                                Inactive Guilds (invite the bot to activate.)
                            </h2>
                            {Array.from(userStore.guilds.values())
                                .filter(i => !i.cached)
                                .map(this.makeGuild(userStore, true))}
                        </div>
                    </ScrollArea>
                </div>
                <div id={styles.SidebarSettings}>
                    <PersonIcon />
                    {userStore.info == null ? (
                        'Not logged in'
                    ) : (
                        <span> signed in as: {userStore.info.username} </span>
                    )}

                    <div style={{ flexGrow: 1 }} />

                    {userStore.info && (
                        <div
                            onClick={this.onLogout(userStore)}
                            id={styles.SidebarSignOut}
                        >
                            Sign out
                        </div>
                    )}
                </div>
            </div>
        );
    }

    private makeGuild(
        userStore: UserStore,
        inactive: boolean,
    ): (value: IGuildInfo, index: number, array: IGuildInfo[]) => JSX.Element {
        const reroute = () => {
            window.location.href = '/';
        };
        return props => (
            <GuildItem
                authed={userStore.hasRolePermissions(props.id)}
                active={props.id === userStore.activeGuild}
                key={props.id}
                onClick={
                    inactive ? reroute : this.onItemClick(userStore, props)
                }
                {...props}
            />
        );
    }

    private onItemClick(userStore: UserStore, props: IGuildInfo) {
        return () => {
            userStore.activeGuild = props.id;

            this.props.userStore!.activeState = 'view';
            this.props.routingStore!.redirect = `/${props.id}/view`;
        };
    }

    private onLogout(
        userStore: UserStore,
    ): ((event: React.MouseEvent<HTMLDivElement>) => void) | undefined {
        return () => {
            userStore.clearTokens();
            window.location.href = '/';
        };
    }
}

export default Sidebar;
