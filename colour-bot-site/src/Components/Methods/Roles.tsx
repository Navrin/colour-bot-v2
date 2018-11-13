import * as React from 'react';
import { GuildStore, IRole } from '../../stores/Guild';
import { inject, observer } from 'mobx-react';
import { stores } from '../..';
import { ColourItem } from './ColourItem';
import { autorun, observable, toJS, IReactionDisposer } from 'mobx';
// import { ScrollArea } from '../Sidebar/Sidebar';
import { UserStore } from '../../stores/User';
import { BaseButton } from '../BaseButton';
import Add from '@material-ui/icons/Add';
import Cancel from '@material-ui/icons/Cancel';
import { mapify } from '../../stores/mapify';
import * as _ from 'lodash';
import ListComponent from './Helpers/ListComponent';
import * as styles from './styles/Roles.module.scss';
import VirtualList from 'react-tiny-virtual-list';

interface IRolesProps {
    id: string;

    guildStore?: GuildStore;
    userStore?: UserStore;
}

interface IRoleState {
    name?: string;
    state?: 'selected';
    role: IRole;
    ref?: ColourItem;
}

@inject((allStore: typeof stores) => ({
    guildStore: allStore.guildStore,
    userStore: allStore.userStore,
}))
@observer
class Roles extends React.Component<IRolesProps> {
    @observable.shallow
    protected roles: Map<string, IRoleState> = new Map();
    endAutorun: IReactionDisposer | null = null;
    listRender: VirtualList | null;

    public async componentDidMount() {
        this.endAutorun = autorun(async () => {
            console.count('roles autorun');

            const id = this.props.userStore!.activeGuild!;
            if (id == null) {
                return;
            }

            const guildStore = this.props.guildStore!;
            const guild = guildStore.guilds.get(id);

            if (guild == null) {
                const guild = await guildStore.addGuildInfo(id);
                if (guild == null) {
                    return;
                }

                const existing = Array.from(guild.colours.values()).map(
                    it => it.id,
                );

                const roles = mapify(
                    Array.from(guild.roles.values())
                        .map(e => ({
                            role: e,
                            id: e.id,
                        }))
                        .filter(it => !existing.includes(it.role.id)),
                );

                this.roles = roles;
                return;
            }

            const roles = mapify(
                Array.from(guild.roles.values()).map(e => ({
                    role: e,
                    id: e.id,
                })),
            );

            this.roles = roles;
        });
    }

    public componentWillUnmount() {
        if (this.endAutorun) {
            this.endAutorun();
        }
    }

    public render() {
        return (
            <ListComponent
                listRef={ref => (this.listRender = ref)}
                rowRender={this.renderRow}
                rowCount={this.roles.size}
                message="Click the name to change it."
            >
                <div className={styles.Buttons}>
                    <BaseButton
                        prompt="Add roles"
                        onClick={async () => {
                            const primedRoles = Array.from(this.roles.values())
                                .filter(it => it.state)
                                .map(it => ({
                                    roleId: it.role.id,
                                    hex: it.role.colour,
                                    name: it.name || it.role.name,
                                }));

                            await this.props.guildStore!.createColours(
                                this.props.id,
                                primedRoles,
                            );

                            this.clearRoles()();
                        }}
                        active="false"
                    />
                    <BaseButton
                        prompt="Clear roles"
                        onClick={this.clearRoles()}
                        active="false"
                    />
                </div>
            </ListComponent>

            // <div id={styles.Root}>
            //     <div className={styles.Roles}>
            //         <div>
            //             <i>click the name to change it</i>
            //         </div>

            //         <div>
            //             {this.roles && (
            //                 <AutoSizer>
            //                     {({ height, width }) => (
            //                         <List
            //                             force={this.roles}
            //                             rowCount={this.roles.size}
            //                             rowHeight={105}
            //                             height={height}
            //                             width={width}
            //                             rowRenderer={this.renderRow}
            //                             ref={e => (this.listRender = e)}
            //                         />
            //                     )}
            //                 </AutoSizer>
            //             )}
            //         </div>

            //         <div>
            //         </div>
            //     </div>
            // </div>
        );
    }

    private renderRow = ({
        key,
        index,
        style,
    }: {
        key: string;
        index: number;
        style: {};
    }): JSX.Element => {
        if (this.roles == null) {
            return <div />;
        }

        console.log(style);

        const { state, role, name } = Array.from(this.roles.values())[index];

        return (
            <div style={{ marginBottom: 15, whiteSpace: 'nowrap' }}>
                <ColourItem
                    {...role}
                    currentIcon={state ? Cancel : Add}
                    name={name || role.name}
                    onIconClick={() => {
                        const inst = this.roles.get(role.id)!;

                        this.roles.set(role.id, {
                            state: inst.state ? undefined : 'selected',
                            role,
                            name,
                        });
                    }}
                    onNameChange={e => {
                        this.roles.set(role.id, {
                            state,
                            role,
                            name: e,
                        });
                    }}
                    guildId={this.props.id}
                    canExpand={false}
                    noUpdateMessage={true}
                />
            </div>
        );
    };

    private clearRoles(): (() => void) &
        ((event: React.MouseEvent<HTMLElement>) => void) {
        return () => {
            const roles = new Map(_.entries(toJS(this.roles)));
            Array.from(this.roles.entries()).forEach(([key, value]) => {
                roles.set(key, {
                    ...value,
                    state: undefined,
                });
            });
            this.roles = roles;
        };
    }
}

export default Roles;
