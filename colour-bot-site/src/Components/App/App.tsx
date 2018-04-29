import * as React from 'react';
import * as styles from './styles.module.scss';

class App extends React.Component {
    public render() {
        return (
            <div id={styles.App}>
                <div id={styles.TopBarGuild}>Guild Here</div>
                <div id={styles.TopBarAction}>Action here</div>

                <div id={styles.Guildbar} />
                <div id={styles.Sidebar} />
                <div id={styles.Main} />
            </div>
        );
    }
}

export default App;
