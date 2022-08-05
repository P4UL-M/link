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
import { useNavigation, useTheme } from '@react-navigation/native';
import { useIsFocused } from '@react-navigation/native';
import { AuthContext } from '../context/AuthContext';
import { AxiosContext } from '../context/AxiosContext';
import { login, refresh } from '../services/auth.service';

export default function LoginScreen() {
    const [email, setEmail] = useState('');
    const [password, setPassword] = useState('');
    const [exists, setExists] = useState(false);
    const authContext = useContext(AuthContext);
    const { publicAxios } = useContext(AxiosContext);

    const nav = useNavigation();
    const isFocused = useIsFocused();

    let emailInput = useRef(null);
    let passwordInput = useRef(null);
    let accountBtn = useRef(null);

    const { colors } = useTheme();
    const styles = stylesheet(colors);

    // force reload on focus of screen
    useEffect(() => {
        if (isFocused) {
            // clear on load
            emailInput.current.clear();
            passwordInput.current.clear();
            setEmail('');
            setPassword('');
            setExists(false);
            refresh(publicAxios, authContext).then((state) => {
                if (state) {
                    nav.navigate('Home');
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
                const query = `query exists($email : String!) {
                    exist(email: $email)
                }`;
                const response = await fetch('http://localhost:3000/graphQL', {
                    method: 'POST',
                    headers: {
                        'Content-Type': 'application/json',
                        Accept: 'application/json',
                    },
                    body: JSON.stringify({
                        query,
                        variables: {
                            email: email,
                        },
                    }),
                });
                const json = await response.json();
                return json.data.exist;
            } catch (error) {
                console.error(error);
            }
        };
        if (/\S+@\S+\.\S+/.test(email) && (await checkEmail())) {
            setExists(true);
            emailInput.current.style = styles.textInput;
            passwordInput.current.focus();
            accountBtn.current.style.display = 'none';
        } else {
            setExists(false);
            emailInput.current.style.borderColor = colors.accent;
            accountBtn.current.style.display = 'inherit';
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

            nav.navigate('Home');
            passwordInput.current.style = styles.textInput;
        } catch (error) {
            passwordInput.current.style.borderColor = 'red';
            console.error(error);
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
                            placeholder="email"
                            style={styles.textInput}
                            onSubmitEditing={() => isRegister()}
                            onChangeText={(text) => setEmail(text)}
                            ref={emailInput}
                        />
                        <TouchableWithoutFeedback
                            onPress={() =>
                                nav.navigate('Register', { email: email })
                            }
                            style={styles.textBtn}
                            ref={accountBtn}
                        >
                            <Text style={styles.textBtn_text}>
                                Create Account ?
                            </Text>
                        </TouchableWithoutFeedback>
                        <TextInput
                            textContentType='password'
                            ref={passwordInput}
                            placeholder="Password"
                            placeholderColor={colors.border}
                            style={[
                                styles.textInput,
                                { display: exists ? 'inherit' : 'none' },
                            ]}
                            secureTextEntry={true}
                            autoFocus={true}
                            onSubmitEditing={() => loginReq()}
                            onChangeText={(text) => setPassword(text)}
                        />
                    </View>
                    <View style={styles.btnContainer}>
                        <Button
                            buttonStyle={styles.btn}
                            disabled={exists ? password == '' : email == ''}
                            title={exists ? 'Sign in' : 'Next'}
                            onPress={() => (exists ? loginReq() : isRegister())}
                        />
                    </View>
                </View>
            </TouchableWithoutFeedback>
        </KeyboardAvoidingView>
    );
}
