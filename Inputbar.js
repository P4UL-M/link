import { Text, TextInput } from "react-native";
import { Component } from "react";

export default class InputBar extends Component {
    state = {
        username: '',
        password: ''
    }
 
    getValues() {
        console.log(this.state.username);
        console.log(this.state.password);
    }
 
 
    render() {
        return (
            <>
                <TextInput
                    placeholder="Enter Username"
                    onChangeText={(text) => this.setState({ username: text })}
                    onSubmitEditing={() => this.passwordInput.focus()}
                />
 
                <TextInput
                    placeholder="Enter Password"
                    secureTextEntry={true}
                    onChangeText={(text) => this.setState({ password: text })}
                    ref={(input) => { this.passwordInput = input; }}
                    onSubmitEditing={() => this.getValues()}
                />
                <Text>{this.state.username}</Text>
                <Text>{this.state.password}</Text>
            </>
        );
    }
}
