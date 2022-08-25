import React, { useState, useRef, useEffect, useContext } from 'react';
import stylesheet from './style';
import {
    Keyboard,
    KeyboardAvoidingView,
    Text,
    TextInput,
    TouchableWithoutFeedback,
    View,
    Platform,
} from 'react-native';
import { Button } from 'react-native-elements';
import { useTheme } from '@react-navigation/native';
import { useIsFocused } from '@react-navigation/native';
import { AuthContext } from '../context/AuthContext';
import { AxiosContext } from '../context/AxiosContext';
import { login, refresh } from '../services/auth.service';
// Import everything needed to use the `useQuery` hook
import { gql, useApolloClient } from '@apollo/client';

export default function LoginScreen({navigation}) {
    const [email, setEmail] = useState('');
    const [password, setPassword] = useState('');
    const [appState, setAppState] = useState({email_valid: false, password_valid: false});
    const authContext = useContext(AuthContext);
    const { publicAxios } = useContext(AxiosContext);

    const isFocused = useIsFocused();
    const passwordRef = useRef(null);

    const { colors } = useTheme();
    const styles = stylesheet(colors);

    const client = useApolloClient();

    // force reload on focus of screen
    useEffect(() => {
        if (isFocused) {
            // clear on load
            setEmail('');
            setPassword('');
            setAppState({email_valid: null, password_valid: null});
            refresh(publicAxios, authContext).then((state) => {
                if (state) {
                    navigation.navigate('Home');
                }
            });
        }
    }, [isFocused]);

    /**
     * @returns {Promise<boolean>}
     */
    async function isRegister() {
        // check if user exists here
        const checkEmail = async () => {
            try {
                const query = gql`query exists($email : String!) {
                    exist(email: $email)
                }`;
                const response = await client.query({
                    query,
                    variables: {
                        email,
                    },
                });
                return response.data.exist;
            } catch (error) {
                console.error(error);
            }
        };
        if (/\S+@\S+\.\S+/.test(email) && (await checkEmail())) {
            setAppState({...appState, email_valid: true});
            passwordRef.current.focus();
        } else {
            setAppState({...appState, email_valid: false});
        }
    }

    async function loginReq() {
        try {
            const response = await publicAxios.post('/auth/login', {
                email,
                password,
            });

            const { access_token, refresh_token } = response.data;

            await login(authContext, access_token, refresh_token);

            navigation.navigate('Home');
        } catch (error) {
            setAppState({...appState, password_valid: false});
        }
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
                    <Text style={styles.header}>Link</Text>
                    <View>
                        <TextInput
                            textContentType='username'
                            autoCorrect={false}
                            autoCapitalize='none'
                            placeholder="email"
                            placeholderTextColor={colors.border}
                            style={[styles.textInput, {
                                borderColor: appState.email_valid === false ? colors.accent : colors.border,
                            }]}
                            onSubmitEditing={() => isRegister()}
                            onChangeText={(text) => setEmail(text)}
                            value={email}
                        />
                        {
                            appState.email_valid === false && email != '' ? <TouchableWithoutFeedback
                                onPress={() =>
                                    navigation.navigate('Register', { email: email })
                                }
                                style={styles.textBtn}
                            >
                                <Text style={styles.textBtn_text}>
                                    Create Account ?
                                </Text>
                            </TouchableWithoutFeedback>: null
                        }
                        <TextInput
                            textContentType='password'
                            ref={passwordRef}
                            placeholder="Password"
                            placeholderTextColor={colors.border}
                            style={[
                                styles.textInput,
                                { 
                                    display: appState.email_valid ? 'flex' : 'none',
                                    borderColor: appState.password_valid === false ? colors.accent : colors.border, 
                                },
                            ]}
                            secureTextEntry={true}
                            autoFocus={true}
                            onSubmitEditing={() => loginReq()}
                            onChangeText={(text) => setPassword(text)}
                            value={password}
                        />
                    </View>
                    <View style={styles.btnContainer}>
                        <Button
                            buttonStyle={styles.btn}
                            disabled={appState.email_valid ? password == '' : email == ''}
                            title={appState.email_valid ? 'Sign in' : 'Next'}
                            onPress={() => (appState.email_valid ? loginReq() : isRegister())}
                        />
                    </View>
                </View>
            </TouchableWithoutFeedback>
        </KeyboardAvoidingView>
    );
}
