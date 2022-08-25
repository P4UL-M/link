import React from 'react';
import stylesheet from './style';
import {
    View
} from 'react-native';
import { Button } from 'react-native-elements';
import { useTheme } from '@react-navigation/native';
import { TextInput } from '../../node_modules/react-native-gesture-handler';

function InputBar() {
    const { colors } = useTheme();
    const styles = stylesheet(colors);

    function handleSubmit() {
        console.log('submit');
    }

    return (
        <View style={styles.container}>
            <TextInput style={styles.input} multiline placeholder="Message" />
            <Button title="Send" onPress={handleSubmit} buttonStyle={styles.btn}/>
        </View>
    );
}

export default InputBar;
