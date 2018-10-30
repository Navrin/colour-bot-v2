import { observable } from 'mobx';

export enum NotificationType {
    SUCCESS,
    FAILURE,
}

export interface INotification {
    message: string;
    type: NotificationType;
}

export class NotificationStore {
    @observable
    public active: INotification | null = null;

    protected timer?: TimerHandler;

    public send(notification: INotification) {
        if (this.timer) setTimeout(this.timer);
        this.active = notification;

        this.timer = setTimeout(() => {
            this.active = null;
        }, 15000) as any;
    }
}

const notificationStore = new NotificationStore();

export default notificationStore;
