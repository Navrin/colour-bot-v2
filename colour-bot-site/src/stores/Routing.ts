import { observable } from 'mobx';

export class RoutingStore {
    @observable
    public redirect?: string;
}
