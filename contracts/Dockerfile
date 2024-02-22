FROM node:20.9.0

WORKDIR /root

RUN npm init -y
RUN npm i hardhat @nomicfoundation/hardhat-toolbox
RUN npm i @openzeppelin/hardhat-upgrades
RUN npm i --save-dev typescript ts-node hardhat-contract-sizer dotenv

COPY ./hardhat.config.ts ./
COPY ./tsconfig.json ./
COPY ./test-accounts.ts ./

EXPOSE 8545

CMD ["npx", "hardhat", "node"]
