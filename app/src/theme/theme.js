import { DefaultTheme } from '@react-navigation/native';

export const Classic = {
    ...DefaultTheme,
    colors: {
        ...DefaultTheme.colors,
        background: '#FDFEFE',
        text: '#201F30',
        primary: '#8685EF',
        secondary: '#EC8776',
        accent: '#D1406C',
        border: '#A49DAA',
    },
};

export const Dark = {
    ...DefaultTheme,
    colors: {
        ...DefaultTheme.colors,
        background: '#201F30',
        text: '#FDFEFE',
        primary: '#8685EF',
        secondary: '#EC8776',
        accent: '#D1406C',
        border: '#FDFEFE',
    },
};
