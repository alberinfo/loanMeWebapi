CREATE TABLE PerfilCrediticio(
    ID BIGSERIAL PRIMARY KEY NOT NULL,
    fkUsuario BIGINT NOT NULL REFERENCES Usuario(ID),
    DNI TEXT UNIQUE NOT NULL,
    HistorialCrediticio TEXT,
    ExtractoBancario TEXT,
    ComprobanteDeIngreso TEXT,
    DescripcionFinanciera TEXT
)