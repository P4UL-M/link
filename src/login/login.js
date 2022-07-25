import React, {useState, useRef, useEffect,useContext } from "react";
import styles from "./style";
import {
    Keyboard,
    KeyboardAvoidingView,
    Text,
    TextInput,
    TouchableWithoutFeedback,
    View,
    Platform,
} from "react-native";
import { Button } from "react-native-elements";
import tw from '../../lib/tailwind'; // or, if no custom config: `from 'twrnc'`
import { useDeviceContext } from 'twrnc';
import { useNavigation } from '@react-navigation/native';
import { useIsFocused } from '@react-navigation/native';
import {AuthContext} from '../context/AuthContext';
import {AxiosContext} from '../context/AxiosContext';
import { storeAuth } from '../store/authStore';

export default function LoginScreen() {
    const [email, setEmail] = useState('');
    const [password, setPassword] = useState('');
    const [exists, setExists] = useState(false);
    const authContext = useContext(AuthContext);
    const {publicAxios} = useContext(AxiosContext);

    useDeviceContext(tw)
    const nav = useNavigation();
    const isFocused = useIsFocused();

    let emailInput = useRef(null);
    let passwordInput = useRef(null);

    // force reload on focus of screen
    useEffect(() => {
        // clear on load
        console.log('login screen focused');
        emailInput.current.clear();
        passwordInput.current.clear();
        setEmail('');
        setPassword('');
        setExists(false);
    } , [isFocused])

    async function isRegister() {
        // check if user exists here
        const checkEmail = async () => {
            try {

                const query = `query exists($email : String!) {
                    exist(email: $email)
                }`;
                const response = await fetch(
                    'http://localhost:3000/graphQL', {
                        method: 'POST',
                        headers: {
                            'Content-Type': 'application/json',
                            Accept: 'application/json',
                        },
                        body: JSON.stringify({
                            query,
                            variables: {
                                email: email
                            },
                        }),
                    }
                );
                const json = await response.json();
                return json.data.exist;
            } catch (error) {
                console.error(error);
            }
        };
        if (/\S+@\S+\.\S+/.test(email) && await checkEmail()) {
            setExists(true);
            emailInput.current.style = styles.textInput;
            passwordInput.current.focus();
        }
        else {
            setExists(false);
            emailInput.current.style.borderColor = "red";
        }
    }

    async function login() {

        try {
            const response = await publicAxios.post('/auth/login', {
                email,
                password,
            });

            const {access_token, refresh_token} = response.data;

            await authContext.setAuthState({
                accessToken: access_token,
                refreshToken: refresh_token,
                authenticated: true}
            );

            storeAuth(access_token, refresh_token);
      
            nav.navigate('InputBar');
            passwordInput.current.style = styles.textInput;
        } catch (error) {
            passwordInput.current.style.borderColor = "red";
            console.error(error);
        }
    }

    return (
        <KeyboardAvoidingView
            behavior={Platform.OS === "ios" ? "padding" : "height"}
            style={styles.container}
        >
            <TouchableWithoutFeedback onPress={Platform.select(
                {native: (Keyboard.dismiss),
                    web: (() => null)})}>
                <View style={[styles.inner, tw` bg-white dark:bg-gray-700`]}>
                    <Text style={styles.header}>Header</Text>
                    <View>
                        <TextInput 
                            placeholder="email" 
                            style={styles.textInput} 
                            onSubmitEditing={() => isRegister()} 
                            onChangeText={(text) => setEmail(text)} 
                            ref={emailInput} 
                        />
                        <TextInput
                            ref={passwordInput}
                            placeholder="Password"
                            placeholderColor="#c4c3cb"
                            style={[styles.textInput, {display: exists ? 'inherit' : 'none'}]}
                            secureTextEntry={true}
                            autoFocus={true}
                            onSubmitEditing={() => login()}
                            onChangeText={(text) => setPassword(text)}
                        /> 
                        
                    </View>
                    <View style={styles.btnContainer}>
                        <Button title="Submit" onPress={() => exists ? login() : isRegister()} />
                    </View>
                </View>
            </TouchableWithoutFeedback>
        </KeyboardAvoidingView>
    );
}
