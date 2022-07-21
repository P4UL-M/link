const React = require("react-native");

const { StyleSheet } = React;

const styles = StyleSheet.create({
    container: {
        flex: 1
    },
    inner: {
        padding: 24,
        flex: 1,
        justifyContent: "space-around"
    },
    header: {
        fontSize: 36,
        marginBottom: 48
    },
    textInput: {
        height: 40,
        borderColor: "#000000",
        borderBottomWidth: 1,
        marginBottom: 10
    },
    btnContainer: {
        backgroundColor: "white",
        marginTop: 12
    },
});
export default styles;
