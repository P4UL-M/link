import { ValidationPipe } from '@nestjs/common';
import { NestFactory } from '@nestjs/core';
import { AppModule } from './app.module';
import { GraphQLError, GraphQLFormattedError } from 'graphql';

async function bootstrap() {
    const app = await NestFactory.create(AppModule);
    app.useGlobalPipes(new ValidationPipe());
    app.enableCors(); //! to remove after testing
    await app.listen(3000);
}
bootstrap();
