import React, {useState, useRef, useEffect} from "react";
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

export default function LoginScreen() {
    const [email, setEmail] = useState('');
    const [password, setPassword] = useState('');
    const [exists, setExists] = useState(false);

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

    function isRegister() {
        if (/\S+@\S+\.\S+/.test(email)) {
            // check if user exists here
            setExists(true);
            console.log("email is valid");
            emailInput.current.style = styles.textInput;
        }
        else {
            console.log("email is invalid :" + email);
            console.log(emailInput);
            emailInput.current.style.borderColor = "red"; 
        }
    }

    function login() {
        // test log with email and password here
        console.log("login : " + email + " " + password);
        nav.navigate('InputBar');
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
