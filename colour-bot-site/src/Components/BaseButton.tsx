import * as React from 'react';
import * as styles from './App/styles.module.scss';
import ButtonBase, { ButtonBaseProps } from '@material-ui/core/ButtonBase';

export function BaseButton(
    props: {
        prompt: string;
        onClick: () => void;
        active: string;
    } & ButtonBaseProps,
) {
    return (
        <ButtonBase
            aria-selected={props.active.toString() as 'true' | 'false'}
            onClick={props.onClick}
            classes={{ root: styles.Button }}
            {...props}
        >
            {props.prompt}
        </ButtonBase>
    );
}
