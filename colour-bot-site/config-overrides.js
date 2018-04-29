//! UNTYPED !//

const { compose } = require('react-app-rewired');
const rewireTypescript = require('react-app-rewire-typescript');
const rewireCssModules = require('react-app-rewire-css-modules');

module.exports = (config, env) => {
    const rewires = compose(rewireTypescript, rewireCssModules, (cfg, _) => {
        cfg.module.rules.push({
            enforce: 'pre',
            exclude: /node_modules/,
            loader: 'typed-css-modules-loader',
            test: /\.module.(scss|css)$/,
        });

        return cfg;
    });

    return rewires(config, env);
};
