declare module 'ntcjs' {
    type RGB = string;
    type Name = string;
    type ExactMatch = boolean;

    function name(input: string): [RGB, Name, ExactMatch];

    export { name };
}
