import { Resolver, Query, Mutation, Args, Subscription } from '@nestjs/graphql';
import { MessagesService } from './messages.service';
import { MessageInput, FilterMessageInput } from './message.input';
import { Message } from './interfaces/message.entity';
import { GqlAuthGuard, CurrentUser } from '../auth/graphql-auth.guard';
import { UseGuards, UnauthorizedException } from '@nestjs/common';
import { User } from '../users/interfaces/user.entity';
import { PubSub } from 'graphql-subscriptions';
import { EventMessage, Action } from './interfaces/event.entity';

const pubSub = new PubSub();
@Resolver(() => Message)
export class MessagesResolver {
    constructor(private readonly messagesService: MessagesService) {}

    @UseGuards(GqlAuthGuard)
    @Query(() => Message, { nullable: true })
    async Message(@Args('id', { type: () => String }) id: string): Promise<Message> {
        return this.messagesService.findOne(id);
    }

    @UseGuards(GqlAuthGuard)
    @Query(() => [Message])
    async Messages(
        @Args('filter', { nullable: true }) filter: FilterMessageInput
    ): Promise<Message[]> {
        return this.messagesService.findAll(filter);
    }

    @UseGuards(GqlAuthGuard)
    @Mutation(() => Message)
    async createMessage(@Args('input') input: MessageInput): Promise<Message> {
        const msg = await this.messagesService.create(input);
        pubSub.publish('channelMessage', {
            ChannelMessage: {
                message: msg,
                type: Action.NEW,
            },
        });
        return msg;
    }

    @UseGuards(GqlAuthGuard)
    @Mutation(() => Message)
    async newMessage(
        @CurrentUser() user: User,
        @Args('content') content: string
    ): Promise<Message> {
        const msg = await this.messagesService.newMessage(user, content);
        pubSub.publish('channelMessage', {
            ChannelMessage: {
                message: msg,
                type: Action.NEW,
            },
        });
        return msg;
    }

    @UseGuards(GqlAuthGuard)
    @Mutation(() => Message, { nullable: true })
    async deleteMessage(@CurrentUser() user: User, @Args('id') id: string): Promise<Message> {
        const msg = await this.messagesService.findOne(id);
        if (!msg) {
            return null;
        } else if (msg.sender._id !== user._id && user.credidential < 8) {
            // check if user can delete message
            throw new UnauthorizedException();
        }
        pubSub.publish('channelMessage', {
            ChannelMessage: {
                message: msg,
                type: Action.DELETE,
            },
        });
        return this.messagesService.delete(id);
    }

    @UseGuards(GqlAuthGuard)
    @Mutation(() => Message, { nullable: true })
    async updateMessage(
        @CurrentUser() user: User,
        @Args('id') id: string,
        @Args('content') content: string
    ): Promise<Message> {
        const msg = await this.messagesService.findOne(id);
        if (!msg) {
            return null;
        } else if (msg.sender._id !== user._id && user.credidential < 8) {
            // check if user can update message
            throw new UnauthorizedException();
        }
        pubSub.publish('channelMessage', {
            ChannelMessage: {
                message: msg,
                type: Action.UPDATE,
            },
        });
        return this.messagesService.update(id, content);
    }

    @Subscription(() => EventMessage)
    async ChannelMessage() {
        return pubSub.asyncIterator('channelMessage');
    }
}
