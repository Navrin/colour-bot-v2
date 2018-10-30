import { observable, action } from 'mobx';
import gql from 'graphql-tag';
import client from '../client';
import notificationStore, { NotificationType } from './Notification';
import { mapify } from './mapify';

export interface IRole {
    id: string;
    name: string;
    colour: string;
    permissions: number;
    hoist: boolean;
}

export interface IColourResponse {
    id: string;
    name: string;
    colour: string;
}

export interface IColourCreateInput {
    name: string;
    hex: string;
    id?: string;
}

export interface IColourUpdateInput {
    name?: string;
    colour?: string;
    id: string;
}

export interface IColourDeleteResponse {
    success: boolean;
    id: string;
}

export interface IGuild {
    roles: Map<ID, IRole>;
    colours: Map<ID, IColourResponse>;
}

export type ID = string;
type GuildMap = Map<ID, IGuild>;

const GUILD_QUERY = gql`
    query GetGuild($id: String!) {
        guild(id: $id) {
            colours {
                id
                name
                colour
            }

            roles {
                id
                name
                colour
                permissions
                hoist
            }
        }
    }
`;

const CREATE_COLOUR_QUERY = gql`
    mutation CreateColour($payload: [ColourCreateInput!]!, $id: String!) {
        createColours(guild: $id, details: $payload) {
            id
            name
            colour
        }
    }
`;
const UPDATE_COLOUR_QUERY = gql`
    mutation UpdateColour($payload: ColourUpdateInput!, $id: String!) {
        updateColour(colourId: $id, newData: $payload) {
            id
            name
            colour
        }
    }
`;
const DELETE_COLOUR_QUERY = gql`
    mutation DeleteColours($payload: [String!]!, $id: String!) {
        deleteColours(guild: $id, ids: $payload) {
            success
            id
        }
    }
`;

export class GuildStore {
    @observable
    public guilds: GuildMap = new Map();

    @action
    public async addGuildInfo(id: string) {
        try {
            if (this.guilds.has(id)) {
                return;
            }

            const result = await client.query<{
                guild: { roles: IRole[]; colours: IColourResponse[] };
            }>({
                query: GUILD_QUERY,
                variables: { id },
            });

            const guild = result.data.guild;

            const colours = mapify(guild.colours);
            const roles = mapify(guild.roles);
            const guildObject = { colours, roles };

            this.guilds.set(id, guildObject);

            return guildObject;
        } catch (e) {
            notificationStore.send({
                message: e.message,
                type: NotificationType.FAILURE,
            });

            return;
        }
    }

    @action
    public async createColour(guildID: string, payload: IColourCreateInput) {
        return this.createColours(guildID, [payload]);
    }

    @action
    public async createColours(guildId: string, payload: IColourCreateInput[]) {
        try {
            const guild = this.guilds.get(guildId);
            if (guild == null) {
                return;
            }

            const result = await client.mutate<{
                createColour: IColourResponse[];
            }>({
                mutation: CREATE_COLOUR_QUERY,
                variables: {
                    id: guildId,
                    payload: payload.map(it => ({
                        hex: it.hex,
                        name: it.name,
                    })),
                },
            });

            const colour = result.data!.createColours;

            this.updateGuild(guild, colour, guildId);

            notificationStore.send({
                message: 'Your colour(s) was created sucessfully!',
                type: NotificationType.SUCCESS,
            });
        } catch (e) {
            notificationStore.send({
                message: e.message,
                type: NotificationType.FAILURE,
            });
        }
    }

    @action
    public async updateColour(guildId: string, payload: IColourUpdateInput) {
        try {
            const guild = this.guilds.get(guildId);

            if (guild == null) {
                return;
            }

            const response = await client.mutate<{
                updateColour: IColourResponse;
            }>({
                mutation: UPDATE_COLOUR_QUERY,
                variables: {
                    id: payload.id,
                    payload: {
                        hex: payload.colour,
                        name: payload.name,
                        updateRoleName: true,
                    },
                },
            });

            const colour = response.data!.updateColour;
            this.updateGuild(guild, colour, guildId);

            notificationStore.send({
                message: 'Your colour was updated sucessfully!',
                type: NotificationType.SUCCESS,
            });
        } catch (e) {
            notificationStore.send({
                message: e.message,
                type: NotificationType.FAILURE,
            });
        }
    }

    @action
    public async deleteColours(guildId: string, payload: string[]) {
        try {
            const guild = this.guilds.get(guildId);

            if (guild == null) {
                return;
            }

            const response = await client.mutate<{
                deleteColours: IColourDeleteResponse[];
            }>({
                mutation: DELETE_COLOUR_QUERY,
                variables: {
                    payload,
                    id: guildId,
                },
            });

            const responses: IColourDeleteResponse[] = response.data!
                .deleteColours;

            responses.filter(it => it.success).map(state => {
                guild.colours.delete(state.id);
            });

            notificationStore.send({
                message: 'Colour was deleted sucessfully!',
                type: NotificationType.SUCCESS,
            });
        } catch (e) {
            notificationStore.send({
                message: e.message,
                type: NotificationType.FAILURE,
            });
        }
    }

    private updateGuild(
        guild: IGuild,
        colourList: IColourResponse[],
        guildId: string,
    ) {
        const colours = guild.colours;

        colourList.forEach(colour => colours.set(colour.id, colour));
        guild.colours = colours;

        this.guilds.set(guildId, guild);
    }
}
