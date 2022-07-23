import { InputType, Field } from '@nestjs/graphql';
import { FilterableField } from '@nestjs-query/query-graphql';
import { IsEmail, IsString, IsDate, IsOptional } from 'class-validator';

@InputType()
export class UserInput {
    @Field(() => String, { description: 'Pseudo of the user' })
    @IsString()
    readonly pseudo: string;
    @Field(() => String, { description: 'E-mail of the user' })
    @IsEmail()
    readonly email: string;
    @Field(() => String, { description: 'Password of the user' })
    @IsString()
    readonly password: string;
    @Field(() => String, { description: 'Public key of the user' })
    @IsString()
    readonly publicKey: string;
    @Field(() => Number, { description: 'access of the user : bin rwrw' })
    readonly credidential: number;
}

@InputType()
export class NewUserInput {
    @Field(() => String, { description: 'Pseudo of the user' })
    @IsString()
    readonly pseudo: string;
    @Field(() => String, { description: 'E-mail of the user' })
    @IsEmail()
    readonly email: string;
    @Field(() => String, { description: 'Password of the user' })
    @IsString()
    readonly password: string;
}

@InputType()
export class SearchUserInput {
    @Field(() => String, { description: 'Pseudo of the user' })
    @IsString()
    readonly pseudo: string;
    @Field(() => String, { description: 'Public key of the user' })
    @IsString()
    readonly publicKey: string;
}

@InputType()
export class UpdateUserInput {
    /*
    @Field(() => String, { nullable: true, description: 'First Name of the user' })
    readonly firstName?: string;
    @Field(() => String, { nullable: true, description: 'Last Name of the user' })
    readonly lastName?: string;
    @IsEmail()
    @IsOptional()
    @Field(() => String, { nullable: true, description: 'E-mail of the user' })
    readonly email?: string;
    @Field(() => String, { nullable: true, description: 'Password of the user' })
    readonly password?: string;
    @IsDate()
    @IsOptional()
    @Field(() => Date, { nullable: true, description: 'Birthdate of the user' })
    readonly birthdate?: Date;*/
}

@InputType()
export class FilterUserInput {
    @FilterableField(() => [String], { nullable: true, description: 'Pseudo of the user' })
    readonly pseudo?: string;
    @FilterableField(() => [String], { nullable: true, description: 'E-mail of the user' })
    readonly email?: [string];
    @FilterableField(() => [String], { nullable: true, description: 'Password of the user' })
    readonly password?: [string];
    @FilterableField(() => [String], { nullable: true, description: 'Public key of the user' })
    readonly publicKey?: [string];
    @FilterableField(() => [Number], {
        nullable: true,
        description: 'access of the user : bin rwrw',
    })
    readonly credidential?: [number];
}
