//! UNTYPED !//

const { compose } = require('react-app-rewired');
const rewireTypescript = require('react-app-rewire-clean-typescript');
const rewireCssModules = require('react-app-rewire-css-modules');

module.exports = (config, env) => {
    const rewireCssTyped = env === 'DEVELOPMENT' ? (cfg, _) => {
        cfg.module.rules.push({
            enforce: 'pre',
            exclude: /node_modules/,
            loader: 'typed-css-modules-loader',
            test: /\.module.(scss|css)$/,
        });

        return cfg;
    } : (cfg) => cfg;

    const rewires = compose(
        rewireTypescript,
        rewireCssModules,
        rewireCssTyped,
    );

    return rewires(config, env);
};
