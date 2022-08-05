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
import { AxiosContext } from '../context/AxiosContext';
import { logout } from '../services/auth.service';
import { useNavigation, useTheme } from '@react-navigation/native';

const Home = () => {
    const authContext = useContext(AuthContext);
    const { authAxios } = useContext(AxiosContext);
    const nav = useNavigation();

    const { colors } = useTheme();
    const styles = stylesheet(colors);

    async function getUser() {
        const response = await authAxios.post('graphql', {
            query: `
                query {
                    whoami {
                        _id,
                        pseudo,
                        email,
                        publicKey
                    }
                }
            `,
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
                                nav.navigate('Login');
                            }}
                            buttonStyle={styles.btn}
                        />
                    </View>
                </View>
            </TouchableWithoutFeedback>
        </KeyboardAvoidingView>
    );
};

export default Home;
