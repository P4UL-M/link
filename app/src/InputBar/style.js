import { StyleSheet } from 'react-native';

const stylesheet = (colors, width) =>
    StyleSheet.create({
        container: {
            //position: 'absolute',
            left: 0,
            bottom: 0,
            right: 0,
            display: 'flex',
            flexDirection: 'row',
            alignItems: 'center',
            marginBottom: 0,
        },
        textInput: {
            height: 40,
            width: width - 80,
            borderColor: colors.border,
            borderRadius: 4,
            borderWidth: 1,
            color: colors.text,
            marginHorizontal: 10,
        },
        btn: {
            backgroundColor: colors.primary,
            height: 40,
        },
    });
export default stylesheet;
