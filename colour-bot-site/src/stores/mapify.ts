import { ID } from './Guild';
export function mapify<
    T extends {
        id: string;
    }
>(arr: T[]): Map<ID, T> {
    return arr.reduce((prev, current) => {
        prev.set(current.id, current);
        return prev;
    }, new Map());
}
