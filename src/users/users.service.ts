import { Injectable } from '@nestjs/common';
import { FilterUserInput } from './input-users.input';
import { InjectRepository } from '@nestjs/typeorm';
import { Repository } from 'typeorm';
import { User } from './interfaces/user.entity';
import { UnauthorizedException } from '@nestjs/common';

@Injectable()
export class UsersService {
    constructor(
        @InjectRepository(User)
        private userRepository: Repository<User>
    ) {}

    async create(createUserDto: User): Promise<User> {
        const createdUser = this.userRepository.create(createUserDto);
        await this.userRepository.insert(createdUser);
        return createdUser;
    }

    async newUser(createUserDto: any): Promise<User> {
        // check here if email isn't already used
        if (await this.userRepository.findOne({ where: { email: createUserDto.email } })) {
            throw new UnauthorizedException('Email already used');
        }
        createUserDto.credidential = 1;
        // generate a 4-char random unique code
        createUserDto.publicKey = Math.random()
            .toString(36)
            .toUpperCase()
            .substr(2, 5)
            .padStart(5, '0');
        const createdUser = this.userRepository.create(createUserDto as User);
        await this.userRepository.insert(createdUser);
        return createdUser;
    }

    async findOne(input: any, type = 'id'): Promise<User> {
        if (type == 'email') {
            return await this.userRepository.findOne({ where: { email: input } });
        }
        if (type == 'input') {
            return await this.userRepository.findOne({
                where: { pseudo: input.pseudo, publicKey: input.publicKey },
            });
        }
        return await this.userRepository.findOne({ where: { _id: input } });
    }
    async findAll(filter: FilterUserInput): Promise<User[]> {
        return await this.userRepository.find({
            where: filter || {}, //TODO correct filter format here
        });
    }
    /*

    async delete(id: string): Promise<UserType> {
        return await this.userRepository.findByIdAndRemove(id);
    }
    */
    async update(id: string, input: any): Promise<User> {
        await this.userRepository.update(id, input);
        return this.userRepository.findOne(id);
    }
}
