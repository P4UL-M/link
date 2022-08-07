import { StyleSheet } from 'react-native';
import { Platform } from 'react-native';

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
            ...Platform.select({
                web: {
                    width: '40%',
                    alignSelf: 'center',
                },
            }),
        },
        header: {
            fontSize: 36,
            marginBottom: 48,
            color: colors.secondary,
        },
        textInput: {
            height: 40,
            borderColor: colors.border,
            borderRadius: 4,
            borderWidth: 1,
            marginBottom: 10,
            paddingLeft: 15,
            ...Platform.select({
                web: {
                    outlineStyle: 'none',
                },
            }),
            color: colors.text,
        },
        btnContainer: {
            backgroundColor: 'white',
            marginTop: 12,
        },
        btn: {
            backgroundColor: colors.primary,
        },
        textBtn: {
            display: 'none',
        },
        textBtn_text: {
            color: colors.primary,
            ...Platform.select({
                web: {
                    cursor: 'pointer',
                },
            }),
        },
    });
export default stylesheet;
