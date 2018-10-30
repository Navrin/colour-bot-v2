import settings from './settings';
import ApolloClient from 'apollo-boost';

const client = new ApolloClient({
    uri: `${settings.endpoint}/graphql`,
    request: async operation => {
        const tok = localStorage.getItem('token');

        const token =
            tok != null && tok != 'undefined'
                ? JSON.parse(tok).accessToken
                : undefined;

        operation.setContext({
            headers: {
                Authorization: token,
            },
        });
    },
});

export default client;
