import * as React from 'react';
import { observer } from 'mobx-react';
import { observable, observe } from 'mobx';
import Edit from '@material-ui/icons/Edit';
import * as styles from './styles/Input.module.scss';
import Popover from '@material-ui/core/Popover';
import * as ntc from 'ntcjs';
import { SketchPicker } from 'react-color';

export interface IInputShape {
    colour: string;
    name: string;
}
export interface IInputProps {
    suggestName?: boolean;
    values?: IInputShape;
    onChange: (e: IInputShape) => void;
}

function NameInput(props: {
    value: string;
    onChange: React.ChangeEventHandler<HTMLInputElement>;
    onRequestName: () => void;
}) {
    return (
        <div className={styles.InnerInput}>
            <div>
                <Edit onClick={props.onRequestName} />
            </div>
            <div>
                <input
                    className={styles.InputElement}
                    value={props.value}
                    onChange={props.onChange}
                    placeholder="Name"
                />
            </div>
        </div>
    );
}

const DEFAULT_VALUES = {
    colour: '#ffffff',
    name: 'White',
};
@observer
class Input extends React.Component<IInputProps> {
    public state: { inputRef: HTMLDivElement | null } = { inputRef: null };

    @observable
    protected values: IInputShape = DEFAULT_VALUES;

    @observable
    protected colourOpen: boolean = false;

    constructor(props: IInputProps) {
        super(props);

        this.values = props.values || DEFAULT_VALUES;

        observe(this.values, () => {
            props.onChange(this.values);
        });
    }

    public render() {
        return (
            <form>
                <div>
                    <NameInput
                        onRequestName={() => {
                            let [, name] = ntc.name(this.values.colour);

                            this.values.name = name;
                        }}
                        value={this.values.name}
                        onChange={e => (this.values.name = e.target.value)}
                    />
                </div>
                <div>
                    <div
                        className={styles.InnerInput}
                        onClickCapture={e => {
                            this.setState({ inputRef: e.currentTarget });
                        }}
                    >
                        <div
                            style={{
                                backgroundColor: this.values.colour,
                            }}
                        />
                        <div>
                            <input
                                disabled={true}
                                className={styles.InputElement}
                                value={this.values.colour}
                                placeholder="Colour"
                                pattern="#?([A-Za-z0-9]{1,2}){3}"
                            />
                            <Popover
                                open={this.state.inputRef != null}
                                onClose={() => {
                                    this.setState({ inputRef: null });
                                }}
                                anchorEl={this.state.inputRef as any}
                                anchorOrigin={{
                                    horizontal: 'center',
                                    vertical: 'bottom',
                                }}
                                transformOrigin={{
                                    horizontal: 'center',
                                    vertical: 'top',
                                }}
                            >
                                <SketchPicker
                                    color={this.values.colour}
                                    onChange={e => {
                                        this.values.colour = e.hex;
                                    }}
                                />
                            </Popover>
                        </div>
                    </div>
                </div>
            </form>
        );
    }
}

export default Input;
