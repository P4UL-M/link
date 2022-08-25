import React from 'react';
import stylesheet from './style';
import {
    View,
    TextInput,
    Dimensions,
} from 'react-native';
import { Button } from 'react-native-elements';
import { useTheme } from '@react-navigation/native';

function InputBar() {
    const width = Dimensions.get('window').width;
    const { colors } = useTheme();
    const styles = stylesheet(colors, width);

    function handleSubmit() {
        console.log('submit');
    }

    return (
        <View style={styles.container}>
            <TextInput style={styles.textInput} placeholder="Message" />
            <Button title="Send" onPress={handleSubmit} buttonStyle={styles.btn}/>
        </View>
    );
}

export default InputBar;
