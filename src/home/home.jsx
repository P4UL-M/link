import React, { useContext } from 'react';
import stylesheet from './style';
import {
    Keyboard,
    KeyboardAvoidingView,
    Text,
    TouchableWithoutFeedback,
    View,
    Platform,
} from 'react-native';
import { Button } from 'react-native-elements';
import { AuthContext } from '../context/AuthContext';
import { logout } from '../services/auth.service';
import { useTheme } from '@react-navigation/native';
import { useApolloClient, gql } from '@apollo/client';
import MyChannel from '../Channel/channel';


export default function Home({navigation}) {
    const authContext = useContext(AuthContext);

    const { colors } = useTheme();
    const styles = stylesheet(colors);
    const client = useApolloClient();

    async function getUser() {
        const response = await client.query({
            query: gql`query whoami {
                whoami {
                    _id,
                    pseudo,
                    email,
                    publicKey
                }
            }`,
            options: {
                context: {
                    headers: {
                        'Authorization': `Bearer ${authContext.token}`,
                    },
                },
            },
        });
        console.log(response);
    }

    return (
        <KeyboardAvoidingView
            behavior={Platform.OS === 'ios' ? 'padding' : 'height'}
            style={styles.container}
        >
            <TouchableWithoutFeedback
                onPress={Platform.select({
                    native: Keyboard.dismiss,
                    web: () => null,
                })}
            >
                <View style={styles.inner}>
                    <Text style={styles.header}>Header</Text>
                    <View style={styles.btnContainer}>
                        <Button title="Submit" onPress={getUser} buttonStyle={styles.btn} />
                    </View>
                    <View style={styles.btnContainer}>
                        <Button
                            title="Logout"
                            onPress={() => {
                                logout(authContext);
                                navigation.navigate('Login');
                            }}
                            buttonStyle={styles.btn}
                        />
                    </View>
                    <MyChannel />
                </View>
            </TouchableWithoutFeedback>
        </KeyboardAvoidingView>
    );
}
