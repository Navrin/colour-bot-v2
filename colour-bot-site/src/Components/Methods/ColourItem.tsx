import * as React from 'react';
import { IColourResponse, GuildStore } from '../../stores/Guild';
import { observer, inject } from 'mobx-react';
import { ColourPreview } from './View';
import { observable, autorun } from 'mobx';
import Input, { IInputShape } from './Input';
import { BaseButton } from '../BaseButton';
import { stores } from '../..';
import { UserStore } from '../../stores/User';
import { InputElement } from './styles/Input.module.scss';
import Add from '@material-ui/icons/Add';
import * as styles from './styles/ColourItem.module.scss';
// const ResizeObserver = require('react-resize-observer').default;

// import * as styles from './styles/View.module.scss';

interface IColourItemProps {
    guildStore?: GuildStore;
    userStore?: UserStore;
    guildId: string;

    permissions?: number;
    canExpand: boolean;
    noUpdateMessage?: boolean;

    currentIcon?: typeof Add;
    onIconClick?: () => void;

    onNameChange?: (e: string) => void;
    onExpand?: (i: boolean) => void;
    onColourStateUpdate?: (it: IInputShape | null) => void;

    style?: {};

    mref?: (e: Element | null) => void;

    isExpanded?: boolean;
    updateState?: IInputShape;
}

@inject((allStores: typeof stores) => ({
    guildStore: allStores.guildStore,
    userStore: allStores.userStore,
}))
@observer
export class ColourItem extends React.Component<
    IColourResponse & IColourItemProps
> {
    @observable
    protected isExpanded: boolean = this.props.isExpanded || false;

    @observable
    protected colourUpdateState: null | IInputShape =
        this.props.updateState || null;

    componentDidMount() {
        autorun(() => {
            this.props.onColourStateUpdate &&
                this.props.onColourStateUpdate(this.colourUpdateState);
        });
    }

    public render() {
        const nameValue = this.colourUpdateState
            ? this.colourUpdateState.name
            : this.props.name;
        return (
            <div
                style={this.props.style}
                className={styles.Colour}
                key={this.props.id}
                ref={this.props.mref}
            >
                <div
                    onClick={this.onExpandClick()}
                    className={styles.ColourRow}
                >
                    <div className={styles.ColourIcon}>
                        {this.props.currentIcon &&
                            React.createElement(this.props.currentIcon, {
                                onClick: (
                                    e: React.MouseEvent<SVGSVGElement>,
                                ) => {
                                    e.preventDefault();
                                    e.stopPropagation();
                                    this.props.onIconClick &&
                                        this.props.onIconClick();
                                },
                            })}
                    </div>
                    {this.props.canExpand ? (
                        nameValue
                    ) : (
                        <input
                            className={InputElement}
                            value={nameValue}
                            onChange={e => {
                                this.props.onNameChange &&
                                    this.props.onNameChange(e.target.value);

                                this.colourUpdateState = {
                                    name: e.target.value,
                                    colour: this.props.colour,
                                };
                            }}
                        />
                    )}

                    {this.colourUpdateState &&
                        !this.props.noUpdateMessage &&
                        ' (not saved.)'}
                    <div style={{ flexGrow: 1 }} />

                    <ColourPreview
                        {...this.props}
                        name={
                            this.colourUpdateState
                                ? this.colourUpdateState.name
                                : this.props.name
                        }
                        colour={
                            this.colourUpdateState
                                ? this.colourUpdateState.colour
                                : this.props.colour
                        }
                    />
                </div>

                {this.props.canExpand && (
                    <div
                        style={{
                            height: this.isExpanded ? 230 : 0,
                            width: '50%',
                            position: 'relative',
                        }}
                        className={styles.ColourExpanded}
                    >
                        {/* <ResizeObserver
                            onResize={(e: any) => {
                                this.props.measure && this.props.measure();
                            }}
                        /> */}
                        <Input
                            values={
                                this.props.updateState || {
                                    colour: this.props.colour,
                                    name: this.props.name,
                                }
                            }
                            onChange={e => {
                                if (this.props.canExpand) {
                                    this.colourUpdateState = e;
                                }
                            }}
                        />

                        <BaseButton
                            disabled={this.colourUpdateState == null}
                            className={styles.ColourExpandedButton}
                            active={(this.colourUpdateState != null).toString()}
                            prompt="Update"
                            onClick={() => {
                                if (this.colourUpdateState == null) {
                                    return;
                                }
                                this.props.guildStore!.updateColour(
                                    this.props.guildId,
                                    {
                                        id: this.props.id,
                                        ...this.colourUpdateState,
                                    },
                                );
                                this.colourUpdateState = null;
                            }}
                        />
                    </div>
                )}
            </div>
        );
    }

    private onExpandClick():
        | ((event: React.MouseEvent<HTMLDivElement>) => void)
        | undefined {
        return () => {
            const userStore = this.props.userStore!;
            if (!userStore.hasRolePermissions(this.props.guildId)) {
                return;
            }

            this.isExpanded = !this.isExpanded;
            this.props.onExpand && this.props.onExpand(this.isExpanded);
        };
    }
}
