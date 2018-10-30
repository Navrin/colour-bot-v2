import * as React from 'react';
import { inject, observer } from 'mobx-react';
import { stores } from '../..';
import { NotificationStore, NotificationType } from '../../stores/Notification';
import Snackbar from '@material-ui/core/Snackbar';
import SnackbarContent from '@material-ui/core/SnackbarContent';
import * as styles from './styles.module.scss';

interface INotificationsProps {
    notificationStore?: NotificationStore;
}

@inject((allStores: typeof stores) => ({
    notificationStore: allStores.notificationStore,
}))
@observer
class Notifications extends React.Component<INotificationsProps> {
    protected static BG_COLOURS = {
        [NotificationType.FAILURE]: styles.FailureNotification,
        [NotificationType.SUCCESS]: styles.SuccessNotification,
    };

    public render() {
        const notifications = this.props.notificationStore!;
        return (
            <div>
                <Snackbar
                    anchorOrigin={{ horizontal: 'right', vertical: 'bottom' }}
                    open={notifications.active != null}
                    onClose={this.clearNotifications(notifications)}
                >
                    <SnackbarContent
                        classes={{
                            root: notifications.active
                                ? Notifications.BG_COLOURS[
                                      notifications.active.type
                                  ]
                                : undefined,
                        }}
                        message={
                            notifications.active && (
                                <span>{notifications.active!.message}</span>
                            )
                        }
                    />
                </Snackbar>
            </div>
        );
    }

    private clearNotifications(
        notifications: NotificationStore,
    ):
        | ((event: React.SyntheticEvent<any>, reason: string) => void)
        | undefined {
        return () => {
            notifications.active = null;
        };
    }
}

export default Notifications;
