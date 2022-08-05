import { StyleSheet } from 'react-native';
import { Platform } from 'react-native';

const styles = StyleSheet.create({
    container: {
        flex: 1,
        backgroundColor: '#FDFEFE',
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
        color: '#EC8776',
    },
    textInput: {
        height: 40,
        borderColor: '#A49DAA',
        borderRadius: 4,
        borderWidth: 1,
        marginBottom: 10,
        paddingLeft: 15,
        outlineStyle: 'none',
    },
    btnContainer: {
        backgroundColor: 'white',
        marginTop: 12,
    },
    btn: {
        backgroundColor: '#8685EF',
    },
    textBtn: {
        display: 'none',
    },
    textBtn_text: {
        color: '#8685EF',
        cursor: 'pointer',
    },
});
export default styles;
