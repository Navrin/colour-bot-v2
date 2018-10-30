import * as React from 'react';
import { GuildStore, IGuild } from '../../stores/Guild';
import Progress from '@material-ui/core/CircularProgress';
import { inject, observer } from 'mobx-react';
import { stores } from '../..';
import * as styles from './styles/View.module.scss';
import { ScrollArea } from '../Sidebar/Sidebar';
import { ColourItem } from './ColourItem';
import { UserStore } from '../../stores/User';
import { autorun, observable, observe } from 'mobx';
import Delete from '@material-ui/icons/Delete';

interface IViewProps {
    id: string;
    guildStore: GuildStore;
    userStore: UserStore;
}

export function ColourPreview(props: { colour: string; name: string }) {
    return (
        <div className={styles.Preview} style={{ color: props.colour }}>
            <div>{props.name}</div>
            <div>{props.name}</div>
        </div>
    );
}

@inject((allStores: typeof stores) => ({
    guildStore: allStores.guildStore,
    userStore: allStores.userStore,
}))
@observer
class View extends React.Component<IViewProps> {
    @observable
    protected guild?: IGuild;

    @observable
    protected hasPerm?: boolean = true;

    public componentDidMount() {
        observe(this.props.userStore!.guilds, () => {
            this.hasPerm = this.props.userStore.hasRolePermissionsForActive;
        });

        autorun(() => {
            const { guilds } = this.props.guildStore!;

            const id = this.props.userStore.activeGuild!;
            const guild = guilds.get(id);
            const guildInfo = this.props.userStore.guilds.get(id);

            this.guild = guild;

            if (guildInfo) {
                this.hasPerm = this.props.userStore!.rolePermissionCheck(
                    guildInfo,
                );
            }
        });
    }
    public render() {
        return (
            <div id={styles.Root}>
                {this.guild ? (
                    <div className={styles.List}>
                        <div>
                            {this.hasPerm && <i>click to update the colour</i>}
                        </div>
                        <ScrollArea>
                            {Array.from(this.guild.colours.values()).map(c => (
                                <ColourItem
                                    currentIcon={Delete}
                                    canExpand={true}
                                    guildId={this.props.id}
                                    key={c.id}
                                    onIconClick={async () => {
                                        try {
                                            await this.props.guildStore!.deleteColours(
                                                this.props.id,
                                                [c.id],
                                            );
                                            this.guild!.colours.delete(c.id);
                                        } catch (e) {
                                            console.error(e);
                                        }
                                    }}
                                    {...c}
                                />
                            ))}
                        </ScrollArea>
                    </div>
                ) : (
                    <Progress style={{ fill: '#fff', color: '#fff' }} />
                )}
            </div>
        );
    }
}

export default View;
