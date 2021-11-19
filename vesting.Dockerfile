FROM projectserum/build:v0.17.0

COPY / /vesting
WORKDIR /vesting

RUN yarn
RUN ./vesting.sh