import gql from 'graphql-tag';
import client from '../client';
import { observable, action, intercept, computed } from 'mobx';
import notificationStore, { NotificationType } from './Notification';
import { mapify } from './mapify';
import { Permissions } from 'discord.js';

interface IUserInfo {
    id: string;
    username: string;
    avatar?: string;
    discriminator: string;
}

interface IToken {
    accessToken: string;
    tokenType: string;
    expiresIn: string;
    refreshToken: string;
    scope: string;
}

export interface IGuildInfo {
    id: string;
    name: string;
    icon?: string;
    owner: boolean;
    permissions: number;
    cached: boolean;
}

const avatarUrl = (id: string, hash: string) =>
    `https://cdn.discordapp.com/avatars/${id}/${hash}.png?size=256`;

const INFO_QUERY = gql`
    query GetMeInfo {
        me {
            info {
                username
                discriminator
                id
                avatar
            }

            guilds(cachedOnly: false) {
                id
                name
                icon
                owner
                permissions
                cached
            }
        }
    }
`;

const TOKEN_QUERY = gql`
    query GetToken($code: String!) {
        token(code: $code) {
            accessToken
            tokenType
            expiresIn
            refreshToken
            scope
        }
    }
`;

type ID = string;

export class UserStore {
    @observable
    public info: IUserInfo | null;
    @observable
    public token: IToken | null;
    @observable
    public guilds: Map<ID, IGuildInfo> = new Map();
    @observable
    public activeGuild: string | null = location.pathname.split('/')[1] || null;
    @observable
    public activeState: string | null = location.pathname.split('/')[2] || null;

    constructor() {
        this.token = null;
        this.info = null;

        const token = localStorage.getItem('token');

        if (token == null || token === 'undefined') {
            return;
        }

        intercept(this, 'token', e => {
            if (e !== undefined) {
                setTimeout(() => {
                    this.requestInfo();
                }, 100);
            }

            return e;
        });

        this.token = JSON.parse(token);
    }

    @action
    public async requestInfo() {
        try {
            const request = await client.query<{
                me: {
                    info: IUserInfo;
                    guilds: IGuildInfo[];
                };
            }>({
                query: INFO_QUERY,
            });

            this.guilds = mapify(request.data.me.guilds);
            this.info = request.data.me.info;

            const avatar = this.info.avatar;
            this.info.avatar = avatar && avatarUrl(this.info.id, avatar);
        } catch (e) {
            notificationStore.send({
                message: e.message,
                type: NotificationType.FAILURE,
            });

            localStorage.clear();
            window.location.href = '/';
        }
    }

    @action
    public async getToken(code: string) {
        try {
            const request = await client.query<{
                token: IToken;
            }>({
                query: TOKEN_QUERY,
                variables: {
                    code,
                },
            });

            this.token = request.data.token;
            this.requestInfo();

            localStorage.setItem('token', JSON.stringify(this.token));
        } catch (e) {
            notificationStore.send({
                message: `${e.message}. Automatically signing out.`,
                type: NotificationType.FAILURE,
            });

            setTimeout(() => {
                localStorage.clear();
                location.pathname = '/';
            }, 5000);
        }
    }

    @action
    public clearTokens() {
        localStorage.removeItem('token');
        localStorage.removeItem('code');

        this.token = null;
        this.info = null;
    }

    @computed
    public get hasRolePermissionsForActive() {
        return this.hasRolePermissions(this.activeGuild!);
    }

    @action
    public hasRolePermissions(guildID: ID) {
        const guild = this.guilds.get(guildID);

        if (guild == null) {
            return false;
        }

        return this.rolePermissionCheck(guild);
    }

    public rolePermissionCheck(guild: IGuildInfo) {
        const permission = new Permissions(guild.permissions);

        return permission.has(Permissions.FLAGS.MANAGE_ROLES!);
    }
}
