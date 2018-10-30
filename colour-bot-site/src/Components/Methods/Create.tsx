import * as React from 'react';
import { GuildStore } from '../../stores/Guild';
import { observer, inject } from 'mobx-react';
import { stores } from '../..';
import Input, { IInputShape } from './Input';
import { observable } from 'mobx';
import * as styles from './styles/Create.module.scss';
import { BaseButton } from '../BaseButton';
import { UserStore } from '../../stores/User';

interface ICreateProps {
    id: string;
    guildStore: GuildStore;
    userStore: UserStore;
}

function DiscordMessagePreview(props: {
    name: string;
    avatar?: string;
    icon?: string;
    colour: string;
}) {
    return (
        <div className={styles.MessageRoot}>
            {[styles.DarkMessage, styles.LightMessage].map(st => (
                <div key={st} className={st}>
                    <div
                        className={styles.MessageAvatar}
                        style={{
                            backgroundImage: `url(${props.avatar ||
                                require('../../discord_avatar.png')})`,
                        }}
                    />
                    <div className={styles.MessageContent}>
                        <strong
                            className={styles.MessageName}
                            style={{ color: props.colour }}
                        >
                            {props.name}
                        </strong>
                        <div className={styles.MessageText}>
                            This is an example message.
                        </div>
                    </div>
                </div>
            ))}
        </div>
    );
}

const DEFAULT_SHAPE = {
    colour: '#F32E6F',
    name: 'Red',
};
@inject((allStores: typeof stores) => ({
    guildStore: allStores.guildStore,
    userStore: allStores.userStore,
}))
@observer
class Create extends React.Component<ICreateProps> {
    @observable
    protected values: IInputShape = DEFAULT_SHAPE;

    public render() {
        const user = this.props.userStore!.info || {
            avatar: undefined,
            id: '',
            username: 'User',
        };

        return (
            <div className={styles.Create}>
                <DiscordMessagePreview
                    avatar={user.avatar}
                    name={user.username}
                    colour={this.values.colour}
                />
                <div className={styles.CreateInputs}>
                    <h1>Create a colour.</h1>
                    <div className={styles.Prompt}>
                        Click the pencil to auto-create a name.
                    </div>
                    <Input
                        onChange={e => (this.values = e)}
                        suggestName={true}
                        values={this.values}
                    />
                    <BaseButton
                        prompt="Create"
                        onClick={() => {
                            this.props.guildStore!.createColour(this.props.id, {
                                name: this.values.name,
                                hex: this.values.colour,
                            });
                        }}
                        active={false}
                        className={styles.CreateInputsButton}
                    />
                </div>
            </div>
        );
    }
}

export default Create;
