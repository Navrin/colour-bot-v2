import * as React from 'react';
import * as styles from '../styles/ListComponent.module.scss';
import * as _ from 'lodash';
import VirtualList from 'react-tiny-virtual-list';
const ResizeObserver = require('react-resize-detector').default;

interface IListComponentProps {
    rowRender: (args: { index: number; style: {} }) => JSX.Element;
    listRef: (ref: VirtualList | null) => void;

    rowCount: number;
    message: string;

    useVirtual?: boolean;

    children?: JSX.Element | JSX.Element[];
}

class ListComponent extends React.Component<IListComponentProps> {
    static defaultProps = {
        useVirtual: true,
    };

    sizes = {};

    rowRender = (args: { index: number; style: {} }) => {
        console.log(this.sizes);

        return (
            <div style={args.style}>
                <ResizeObserver
                    handleHeight={true}
                    onResize={(rect: number) => {
                        console.log('resize: ', rect);
                        this.sizes[args.index] = rect;
                    }}
                >
                    {this.props.rowRender(args)}
                </ResizeObserver>
            </div>
        );
    };

    render() {
        console.log(this.props);
        return (
            <div className={styles.Root}>
                <div className={styles.Inner}>
                    <div>
                        <i>{this.props.message}</i>
                    </div>

                    <div>
                        <VirtualList
                            height={300}
                            width="100%"
                            itemCount={this.props.rowCount}
                            itemSize={160}
                            renderItem={this.rowRender}
                            ref={this.props.listRef}
                        />
                    </div>
                    {this.props.children}
                </div>
            </div>
        );
    }
}

export default ListComponent;
