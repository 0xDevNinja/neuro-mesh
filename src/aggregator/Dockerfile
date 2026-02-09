FROM node:18-alpine
WORKDIR /app
COPY package.json package-lock.json* tsconfig.json jest.config.js ./
RUN npm install && npm cache clean --force
COPY src ./src
RUN npm run build
CMD ["node", "dist/index.js"]