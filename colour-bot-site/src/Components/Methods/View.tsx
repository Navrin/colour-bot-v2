import * as React from 'react';
import { GuildStore, IGuild } from '../../stores/Guild';
import Progress from '@material-ui/core/CircularProgress';
import { inject, observer } from 'mobx-react';
import { stores } from '../..';
import * as styles from './styles/View.module.scss';
import { ColourItem } from './ColourItem';
import { UserStore } from '../../stores/User';
import { autorun, observable, observe, IReactionDisposer, Lambda } from 'mobx';
import Delete from '@material-ui/icons/Delete';
import ListComponent from './Helpers/ListComponent';
import { List, ListRowProps } from 'react-virtualized';
import { IInputShape } from './Input';

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

    itemState: {
        [key: string]: { input: IInputShape | undefined; expanded: boolean };
    } = {};

    disposers: (IReactionDisposer | Lambda)[] = [];

    listRef: List | null = null;

    public componentDidMount() {
        this.disposers.push(
            observe(this.props.userStore!.guilds, () => {
                this.hasPerm = this.props.userStore.hasRolePermissionsForActive;
            }),
        );

        this.disposers.push(
            autorun(() => {
                console.count('view autoruns');
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
            }),
        );
    }

    public componentWillUnmount() {
        this.disposers.forEach(it => it());
    }

    public render() {
        return this.guild ? (
            <ListComponent
                rowCount={this.guild.colours.size}
                rowRender={this.rowRender}
                listRef={e => {
                    this.listRef = e;
                }}
                message="Click a colour to update it."
            />
        ) : (
            <Progress style={{ fill: '#fff', color: '#fff' }} />
        );
    }

    private rowRender = (
        props: ListRowProps,
        mref: (r: Element | null) => void,
    ) => {
        if (this.guild == null) {
            return <div />;
        }

        const c = Array.from(this.guild.colours.values())[props.index];

        if (this.itemState[c.id] == null) {
            this.itemState[c.id] = { expanded: false, input: undefined };
        }

        const state = this.itemState[c.id];

        return (
            <ColourItem
                {...c}
                style={{
                    ...props.style,
                    height: 'auto',
                    marginTop: 10,
                    marginBottom: 10,
                }}
                key={props.key}
                mref={mref}
                currentIcon={Delete}
                canExpand={true}
                guildId={this.props.id}
                isExpanded={state.expanded}
                onExpand={i => {
                    state.expanded = i;
                }}
                updateState={state.input}
                onColourStateUpdate={i => {
                    state.input = i || undefined;
                }}
                // colour={state.input ? state.input.colour : undefined}
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
            />
        );
    };
}

export default View;
