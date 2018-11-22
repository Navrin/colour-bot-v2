import * as React from 'react';
import * as styles from '../styles/ListComponent.module.scss';
import * as _ from 'lodash';
import { List, AutoSizer } from 'react-virtualized';
import { observable } from 'mobx';
import { observer } from 'mobx-react';
import Measure from 'react-measure';

interface IListComponentProps {
    rowRender: (
        args: { index: number; style: {} },
        ref: (r: Element | null) => void,
    ) => JSX.Element;
    listRef: (ref: List | null) => void;

    rowCount: number;
    message: string;

    useVirtual?: boolean;

    children?: JSX.Element | JSX.Element[];
}

@observer
class ListComponent extends React.Component<IListComponentProps> {
    static defaultProps = {
        useVirtual: true,
    };

    listRef: List | null = null;

    @observable
    sizes = {};

    rowRender = (args: {
        key: string;
        index: number;
        style: { [key: string]: any };
    }) => {
        return (
            <Measure
                key={args.key}
                bounds={true}
                onResize={rect => {
                    this.sizes[args.index] = rect.bounds!.height + 20;

                    this.listRef && this.listRef.recomputeRowHeights();
                }}
            >
                {({ measureRef }) =>
                    this.props.rowRender(
                        {
                            index: args.index,
                            style: {
                                left: args.style.left,
                                position: args.style.position,
                                top: args.style.top,
                            },
                        },
                        measureRef,
                    )
                }
            </Measure>
        );
    };

    render() {
        return (
            <div className={styles.Root}>
                <div className={styles.Inner}>
                    <div>
                        <i>{this.props.message}</i>
                    </div>

                    <div>
                        <AutoSizer>
                            {({ width, height }) => (
                                <List
                                    height={height}
                                    width={width}
                                    rowCount={this.props.rowCount}
                                    rowHeight={i => this.sizes[i.index] || 120}
                                    rowRenderer={this.rowRender}
                                    ref={ref => {
                                        this.listRef = ref;
                                        this.props.listRef(ref);
                                    }}
                                />
                            )}
                        </AutoSizer>
                    </div>
                    {this.props.children}
                </div>
            </div>
        );
    }
}

export default ListComponent;
