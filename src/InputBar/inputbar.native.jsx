import React, { useState, useContext } from 'react';
import stylesheet from './style';
import {
    View,
    TextInput,
    Dimensions,
} from 'react-native';
import { Button } from 'react-native-elements';
import { useTheme } from '@react-navigation/native';
import { useApolloClient, gql } from '@apollo/client';
import { AuthContext } from '../context/AuthContext';

function InputBar() {
    const width = Dimensions.get('window').width;
    const { colors } = useTheme();
    const styles = stylesheet(colors, width);
    const client = useApolloClient();

    const authContext = useContext(AuthContext);

    const [text, setText] = useState('');

    async function handleSubmit() {
        if (text.length > 0) {
            await client.mutate({
                mutation: gql`mutation newMessage($message: String!) {
                    newMessage(
                        content: $message
                    ) {
                        _id
                        content
                        date
                    }
                }`,
                variables: {
                    message: text,
                },
                options: {
                    context: {
                        headers: {
                            'Authorization': `Bearer ${authContext.getAccessToken()}`,
                        },
                    },
                },
            });
            setText('');
        }
    }

    return (
        <View style={styles.container}>
            <TextInput style={styles.textInput} placeholder="Message" value={text} onChangeText={(input) => setText(input)} onSubmitEditing={handleSubmit} />
            <Button title="Send" onPress={handleSubmit} buttonStyle={styles.btn} disabled={!authContext.authState.authenticated}/>
        </View>
    );
}

export default InputBar;