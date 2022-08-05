import { StyleSheet } from 'react-native';

const stylesheet = (colors) =>
    StyleSheet.create({
        container: {
            flex: 1,
            backgroundColor: colors.background,
        },
        inner: {
            padding: 24,
            flex: 1,
            justifyContent: 'space-around',
        },
        header: {
            fontSize: 36,
            marginBottom: 48,
        },
        textInput: {
            height: 40,
            borderColor: colors.border,
            borderRadius: 4,
            borderWidth: 1,
            marginBottom: 10,
            paddingLeft: 15,
            outlineStyle: 'none',
        },
        btnContainer: {
            backgroundColor: colors.background,
            marginTop: 12,
        },
        btn: {
            backgroundColor: colors.primary,
        },
    });
export default stylesheet;
