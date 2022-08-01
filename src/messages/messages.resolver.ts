import { Resolver, Query, Mutation, Args } from '@nestjs/graphql';
import { MessagesService } from './messages.service';
import { MessageInput } from './message.input';
import { Message } from './interfaces/message.entity';
import { GqlAuthGuard, CurrentUser } from '../auth/graphql-auth.guard';
import { UseGuards } from '@nestjs/common';

@Resolver((of) => Message)
export class MessagesResolver {
    constructor(private readonly messagesService: MessagesService) {}

    /*
    @UseGuards(GqlAuthGuard)
    @Query((returns) => User, { nullable: true })
    async User(
        @Args('id', { type: () => String, nullable: true }) id: string,
        @Args('email', { type: () => String, nullable: true }) email: string,
        @Args('input', { type: () => SearchUserInput, nullable: true })
        input: any
    ): Promise<User> {
        if (id) {
            return this.usersService.findOne(id);
        }
        if (email) {
            return this.usersService.findOne(email, 'email');
        }
        if (input) {
            return this.usersService.findOne(input, 'input');
        }
        return null;
    }

    @UseGuards(GqlAuthGuard)
    @Query((returns) => User, { nullable: true })
    async whoami(@CurrentUser() user: User): Promise<User> {
        return this.usersService.findOne(user._id);
    }

    @Query((returns) => Boolean)
    async exist(@Args('email', { type: () => String }) _email: string): Promise<boolean> {
        return (await this.usersService.findOne(_email, 'email')) != null;
    }

    @UseGuards(GqlAuthGuard)
    @Query((returns) => [User])
    async Users(@Args('filter', { nullable: true }) filter: FilterUserInput): Promise<User[]> {
        return this.usersService.findAll(filter);
    }

    @UseGuards(GqlAuthGuard)
    @Mutation((returns) => User)
    async createUser(@Args('input') input: UserInput): Promise<User> {
        return this.usersService.create(input as User);
    }

    @Mutation((returns) => User)
    async newUser(@Args('input') input: NewUserInput): Promise<User> {
        return this.usersService.newUser(input);
    }*/
}
