import * as React from 'react';
import { observer, inject } from 'mobx-react';
import { UserStore, IGuildInfo } from '../../stores/User';
import { stores } from '../..';
import { observable, observe } from 'mobx';
import { BaseButton } from '../BaseButton';
import { NAVIGATIONS } from './Content';
@inject((allStores: typeof stores) => ({
    userStore: allStores.userStore,
}))
@observer
export class NavigationButtons extends React.Component<{
    name: string;
    onButtonNavigate(str: string): () => void;
    userStore?: UserStore;
}> {
    @observable
    guild: IGuildInfo | null = null;
    public componentDidMount() {
        observe(this.props.userStore!, () => {
            const id = this.props.userStore!.activeGuild;
            if (id == null) {
                return;
            }
            const guild = this.props.userStore!.guilds.get(id);
            if (guild == null) {
                return;
            }
            this.guild = guild;
            this.setState({});
        });
    }
    public render() {
        const auth = this.guild
            ? this.props.userStore!.hasRolePermissions(this.guild.id)
            : true;
        return Object.entries(NAVIGATIONS)
            .filter(([, nav]) => {
                return nav.auth !== true ? true : nav.auth === auth;
            })
            .map(([k, { name: n }]) => (
                <BaseButton
                    key={n}
                    active={k == this.props.userStore!.activeState}
                    onClick={this.props.onButtonNavigate(k)}
                    prompt={n}
                />
            ));
    }
}
