import React, { useState, useRef, useEffect, useContext } from 'react';
import styles from './style';
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
import { useNavigation } from '@react-navigation/native';
import { useIsFocused } from '@react-navigation/native';
import { AuthContext } from '../context/AuthContext';
import { AxiosContext } from '../context/AxiosContext';
import { login, refresh } from '../services/auth.service';

export default function RegisterScreen(ctx) {
    const [email, setEmail] = useState(ctx.route.params.email);
    const [password, setPassword] = useState('');
    const [pseudo, setPseudo] = useState('');
    const [vPassword, setVPassword] = useState('');
    const authContext = useContext(AuthContext);
    const { publicAxios } = useContext(AxiosContext);

    const nav = useNavigation();
    const isFocused = useIsFocused();

    let emailInput = useRef(null);
    let pseudoInput = useRef(null);
    let passwordInput = useRef(null);
    let vPasswordInput = useRef(null);

    // force reload on focus of screen
    useEffect(() => {
        console.log('email is ', email);
        if (isFocused) {
            emailInput.current.value = email;
            setPassword('');
        }
    }, [isFocused]);

    useEffect(() => {
        if (vPassword == password) {
            vPasswordInput.current.style = styles.textInput;
        } else {
            vPasswordInput.current.style.borderColor = 'red';
        }
    }, [password, vPassword]);

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
        if (!/\S+@\S+\.\S+/.test(email) || (await checkEmail())) {
            emailInput.current.style.borderColor = 'red';
            return true;
        } else {
            emailInput.current.style = styles.textInput;
            return false;
        }
    }

    async function register() {
        if (
            email &&
            password &&
            pseudo &&
            !(await isRegister()) &&
            vPassword == password
        ) {
            try {
                const response = await publicAxios.post('graphQL', {
                    query: `mutation newUser($email : String!, $password : String!, $pseudo : String!) {
                            newUser(input: {
                                email: $email,
                                password: $password,
                                pseudo: $pseudo,
                            }) {
                                _id,
                            }
                        } 
                    `,
                    variables: {
                        email: email,
                        password: password,
                        pseudo: pseudo,
                    },
                });
                if (response.data.data.newUser) {
                    const response = await publicAxios.post('/auth/login', {
                        email,
                        password,
                    });

                    const { access_token, refresh_token } = response.data;

                    await login(authContext, access_token, refresh_token);
                    nav.navigate('Login');
                }
            } catch (error) {
                console.error(error);
            }
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
                            placeholder="email"
                            style={styles.textInput}
                            onSubmitEditing={() => {
                                isRegister();
                                pseudoInput.current.focus();
                            }}
                            onChangeText={(text) => setEmail(text)}
                            ref={emailInput}
                        />
                        <TextInput
                            placeholder="Pseudo"
                            placeholderColor="#c4c3cb"
                            style={[styles.textInput]}
                            onChangeText={(text) => setPseudo(text)}
                            onSubmitEditing={() => {
                                passwordInput.current.focus();
                            }}
                            ref={pseudoInput}
                        />
                        <TextInput
                            placeholder="Password"
                            placeholderColor="#c4c3cb"
                            style={[styles.textInput]}
                            secureTextEntry={true}
                            onChangeText={(text) => setPassword(text)}
                            onSubmitEditing={() => {
                                vPasswordInput.current.focus();
                            }}
                            ref={passwordInput}
                        />
                        <TextInput
                            ref={vPasswordInput}
                            placeholder="Confirm Password"
                            placeholderColor="#c4c3cb"
                            style={[styles.textInput]}
                            secureTextEntry={true}
                            onChangeText={(text) => setVPassword(text)}
                            onSubmitEditing={() => {
                                register();
                            }}
                        />
                    </View>
                    <View style={styles.btnContainer}>
                        <Button
                            buttonStyle={styles.btn}
                            disabled={
                                email == '' ||
                                password == '' ||
                                pseudo == '' ||
                                vPassword != password
                            }
                            title={'Sign up'}
                            onPress={() => register()}
                        />
                        <TouchableWithoutFeedback
                            onPress={() => nav.navigate('Login')}
                            style={styles.textBtn}
                        >
                            <Text style={styles.textBtn_text}>Cancel</Text>
                        </TouchableWithoutFeedback>
                    </View>
                </View>
            </TouchableWithoutFeedback>
        </KeyboardAvoidingView>
    );
}
