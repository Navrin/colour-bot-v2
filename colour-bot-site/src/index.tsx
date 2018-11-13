/// <reference path="./index.d.ts" />

import * as React from 'react';
import * as ReactDOM from 'react-dom';
import App from './Components/App';

import './index.scss';
import registerServiceWorker from './registerServiceWorker';
import 'react-virtualized/styles.css';

import { Provider } from 'mobx-react';
import { UserStore } from './stores/User';
import { BrowserRouter, Route, Redirect, Switch } from 'react-router-dom';
import * as queryString from 'query-string';
import { GuildStore } from './stores/Guild';
import { RoutingStore } from './stores/Routing';
import notificationStore from './stores/Notification';

export const stores = {
    notificationStore,
    guildStore: new GuildStore(),
    routingStore: new RoutingStore(),
    userStore: new UserStore(),
};

const callbackRender = () => {
    const { code } = queryString.parse(location.search);
    if (code == null) {
        console.error('Callback received but got no code.');
    } else {
        localStorage.setItem('code', code);
    }
    const to = localStorage.getItem('last-area') || '/';
    localStorage.removeItem('last-area');
    return <Redirect to={to} />;
};

ReactDOM.render(
    <Provider {...stores}>
        <BrowserRouter>
            <Switch>
                <Route
                    path="/callback"
                    exactly={true}
                    render={callbackRender}
                />

                <Route exactly={true} path="/" component={App} />
            </Switch>
        </BrowserRouter>
    </Provider>,
    document.getElementById('root') as HTMLElement,
);
registerServiceWorker();
