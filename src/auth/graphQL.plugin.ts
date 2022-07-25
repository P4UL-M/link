const setHttpPlugin = {
    async requestDidStart() {
        return {
            async willSendResponse({ response }) {
                response.http.headers.set('Custom-Header', 'hello');
                if (response?.errors?.[0]?.message === 'Unauthorized') {
                    response.http.status = 401;
                }
            },
        };
    },
};

export { setHttpPlugin };
