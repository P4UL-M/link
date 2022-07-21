import { FieldMiddleware, MiddlewareContext, NextFn } from '@nestjs/graphql';

export const loggerMiddleware: FieldMiddleware = async (ctx: MiddlewareContext, next: NextFn) => {
    const value = await next();

    const _id = ctx.context.req.user._id;
    const _creditential = ctx.context.req.user.credidential;

    if ((_creditential > 3 && ctx.source._id != _id) || _creditential > 0) {
        return value;
    }
    return 'Unauthorized';
};
