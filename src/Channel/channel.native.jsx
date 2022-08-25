import React, { useEffect, useContext, useState, useRef } from 'react';
import {
    View,
    Text,
    FlatList,
    ActivityIndicator,
    TouchableOpacity
} from 'react-native';
import { useIsFocused } from '@react-navigation/native';
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
    const [messages, setMessage] = useState(
    );
    // ref to the flatlist
    const flatListRef = useRef(null);

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
        if (response.data.Messages) {
            const _messages = response.data.Messages.slice().reverse();
            setMessage([..._messages]);
        }
    }

    useEffect(() => {
        if (isFocused && authContext.authState.authenticated) {
            console.log('isFocused');
            getMessages();
        }
    } , [authContext.authState]);

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
                <TouchableOpacity>
                    <Text>{item?.sender.pseudo}</Text>
                    <Text>{item?.content}</Text>
                </TouchableOpacity>
            )}
            keyExtractor={item => item._id}
            ListEmptyComponent={() => (
                <View>
                    <ActivityIndicator size="large" color="#0000ff" />
                </View>
            )}
            extraData={messages}
            style={{flexGrow: 1, flex:1}}
            ref={flatListRef}
            onLayout={() => {
                if (flatListRef.current && messages) {
                    flatListRef.current.scrollToEnd({animated: true});
                }
            } }
            onContentSizeChange={() => {
                if (flatListRef.current && messages) {
                    flatListRef.current.scrollToEnd({animated: true});
                }
            } }
        />
    );
}

export default MyChannel;