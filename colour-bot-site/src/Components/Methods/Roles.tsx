import * as React from 'react';
import { GuildStore, IRole } from '../../stores/Guild';
import { inject, observer } from 'mobx-react';
import { stores } from '../..';
import { ColourItem } from './ColourItem';
import { autorun, observable, toJS } from 'mobx';
import * as styles from './styles/Roles.module.scss';
import { ScrollArea } from '../Sidebar/Sidebar';
import { UserStore } from '../../stores/User';
import { BaseButton } from '../BaseButton';
import Add from '@material-ui/icons/Add';
import Cancel from '@material-ui/icons/Cancel';
import { mapify } from '../../stores/mapify';
import * as _ from 'lodash';

interface IRolesProps {
    id: string;

    guildStore?: GuildStore;
    userStore?: UserStore;
}

interface IRoleState {
    name?: string;
    state?: 'selected';
    role: IRole;
}

@inject((allStore: typeof stores) => ({
    guildStore: allStore.guildStore,
    userStore: allStore.userStore,
}))
@observer
class Roles extends React.Component<IRolesProps> {
    @observable.shallow
    protected roles: Map<string, IRoleState> = new Map();

    public async componentDidMount() {
        autorun(async () => {
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

    public render() {
        return (
            <div className={styles.Roles}>
                <div>
                    <span>Click the name to change it.</span>
                </div>
                <ScrollArea>
                    {[...this.roles.values()].map(({ state, role, name }) => (
                        <ColourItem
                            {...role}
                            currentIcon={state ? Cancel : Add}
                            key={role.id}
                            name={name || role.name}
                            onIconClick={() => {
                                this.roles.set(role.id, {
                                    state: this.roles.get(role.id)!.state
                                        ? undefined
                                        : 'selected',
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
                        />
                    ))}
                </ScrollArea>
                <div>
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
                        active={false}
                    />
                    <BaseButton
                        prompt="Clear roles"
                        onClick={this.clearRoles()}
                        active={false}
                    />
                </div>
            </div>
        );
    }

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
