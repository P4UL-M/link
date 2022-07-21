import { Injectable } from '@nestjs/common';
import { UserInput, UpdateUserInput, FilterUserInput } from './input-users.input';
import { InjectRepository } from '@nestjs/typeorm';
import { Repository } from 'typeorm';
import { User } from './interfaces/user.entity';

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

    /*
    async findAll(): Promise<UserType[]> {
        return await this.userRepository.find().exec();
    }*/

    async findOne(id: string, email = false): Promise<User> {
        if (email) {
            return await this.userRepository.findOne({ where: { email: id } });
        }
        return await this.userRepository.findOne({ where: { _id: id } });
    }
    /*
    async findByCriteria(filter: FilterUserInput): Promise<UserType[]> {
        return await this.userRepository.find(filter).exec();
    }

    async delete(id: string): Promise<UserType> {
        return await this.userRepository.findByIdAndRemove(id);
    }

    async update(id: string, user: User): Promise<UserType> {
        return await this.userRepository.findByIdAndUpdate(id, user as User, { new: true });
    }*/
}
