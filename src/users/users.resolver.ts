import { Resolver, Query, Mutation, Args } from '@nestjs/graphql';
import { UsersService } from './users.service';
import { UserInput, UpdateUserInput, FilterUserInput } from './input-users.input';
import { User } from './interfaces/user.entity';
import { GqlAuthGuard, CurrentUser } from '../auth/graphql-auth.guard';
import { UseGuards } from '@nestjs/common';

@Resolver((of) => User)
export class UsersResolver {
    constructor(private readonly usersService: UsersService) {}
    /*
    @UseGuards(GqlAuthGuard)
    @Query((returns) => [UserType])
    async Users(): Promise<UserType[]> {
        return this.usersService.findAll();
    }
    */
    @UseGuards(GqlAuthGuard)
    @Query((returns) => User, { nullable: true })
    async User(@Args('id') id: string): Promise<User> {
        return this.usersService.findOne(id);
    }

    @UseGuards(GqlAuthGuard)
    @Query((returns) => User, { nullable: true })
    async whoami(@CurrentUser() user: User): Promise<User> {
        return this.usersService.findOne(user._id);
    }
    /*
    @UseGuards(GqlAuthGuard)
    @Query((returns) => [UserType])
    async UserByCriteria(@Args('filter') filter: FilterUserInput): Promise<UserType[]> {
        return this.usersService.findByCriteria(filter);
    }
    */

    @Mutation((returns) => User)
    async createUser(@Args('input') input: UserInput): Promise<User> {
        return this.usersService.create(input);
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
