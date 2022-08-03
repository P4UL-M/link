/* eslint-disable @typescript-eslint/no-unused-vars */
import { Resolver, Query, Mutation, Args } from '@nestjs/graphql';
import { UsersService } from './users.service';
import { UserInput, NewUserInput, FilterUserInput, SearchUserInput } from './user.input';
import { User } from './interfaces/user.entity';
import { GqlAuthGuard, CurrentUser } from '../auth/graphql-auth.guard';
import { UseGuards } from '@nestjs/common';

@Resolver((of) => User)
export class UsersResolver {
    constructor(private readonly usersService: UsersService) {}

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
    }
    /*
    @UseGuards(GqlAuthGuard)
    @Mutation((returns) => UserType, { nullable: true })
    async updateUser(
        @Args('id') id: string,
        @Args('input') input: UpdateUserInput
    ): Promise<UserType> {
        return this.usersService.update(id, input as User);
    }

    @UseGuards(GqlAuthGuard)
    @Mutation((returns) => UserType, { nullable: true })
    async deleteUser(@Args('id') id: string): Promise<UserType> {
        return this.usersService.delete(id);
    }

    @UseGuards(GqlAuthGuard)
    @Mutation((returns) => [UserType])
    async deleteByCriteria(@Args('filter') filter: FilterUserInput): Promise<UserType[]> {
        const result = await this.usersService.findByCriteria(filter);
        result.forEach((element) => {
            this.usersService.delete(element.id);
        });
        return result;
    }*/
}
