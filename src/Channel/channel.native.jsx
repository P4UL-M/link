import React, { useEffect, useContext, useState } from 'react';
import stylesheet from './style';
import {
    View,
    Text,
    FlatList,
} from 'react-native';
import { useTheme, useIsFocused } from '@react-navigation/native';
import { useSubscription, gql, useApolloClient } from '@apollo/client';
import { AuthContext } from '../context/AuthContext';

const SUB = gql`subscription {
        ChannelMessage {
            message {
                _id
                content
                sender {
                    _id
                    pseudo
                }
                date
            }
        type
        }
    }
`;

function MyChannel() {
    const [messages, setMessage] = useState([
        {
            _id: '1',
            sender: {
                pseudo: 'John Doe',
                _id: '1',
            },
            content: 'Hello World',
            date: '2020-01-01',
        },
    ]);

    const client = useApolloClient();
    const isFocused = useIsFocused();
    const authContext = useContext(AuthContext);

    async function getMessages() {
        const response = await client.query({
            query: gql`query {
                Messages {
                    _id
                    content
                    sender {
                        _id
                        pseudo
                    }
                    date
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
        setMessage([...messages,...response.data.Messages.slice().reverse()]);
    }

    useEffect(() => {
        if (isFocused) {
            getMessages();
        }
    } , [authContext.authState]);

    const { colors } = useTheme();
    const styles = stylesheet(colors);

    const {data, loading, error} = useSubscription(SUB);

    useEffect(() => {
        if (data) {
            setMessage([...messages, data.ChannelMessage.message]);
        }
    } , [data, loading, error]);

    return (
        <FlatList
            data={messages}
            renderItem={({item}) => (
                <View>
                    <Text>{item.sender.pseudo}</Text>
                    <Text>{item.content}</Text>
                </View>
            )}
            keyExtractor={item => item._id}
        />
    );
}

export default MyChannel;