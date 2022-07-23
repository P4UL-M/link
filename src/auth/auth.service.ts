import { Injectable } from '@nestjs/common';
import { UsersService } from '../users/users.service';
import { JwtService } from '@nestjs/jwt';

@Injectable()
export class AuthService {
    constructor(private usersService: UsersService, private jwtService: JwtService) {}

    async validateUser(email: string, pass: string): Promise<any> {
        const user = await this.usersService.findOne(email, 'email');
        if (user && user.password === pass) {
            const result = user;
            return result;
        }
        return null;
    }

    async login(user: any) {
        const payload = { username: user.email, sub: user._id, credidential: user.credidential };
        return {
            access_token: this.jwtService.sign(payload),
        };
    }
}
