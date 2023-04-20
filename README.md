# PETScroll :scroll:
Um aplicação via linha de comando para automatizar alguns processos
relacionados á emissão de certificados referentes a eventos organizados pelo
PET.

```
A simple certificate manager for PETComp events.

Usage: pet-scroll [OPTIONS] --event <EVENT> --atts <ATTENDEES> --output <OUTPUT> <--cert-img <CERT_IMG>|--upload-img <UPLOAD_IMG>>

Options:
  -e, --event <EVENT>            Event Data CSV file
  -a, --atts <ATTENDEES>         Attendees Info CSV file
  -c, --cert-img <CERT_IMG>      An already uploaded event certificate image
  -u, --upload-img <UPLOAD_IMG>  Uploads the given event certificate image to the SFTP server
  -o, --output <OUTPUT>          SQL output file
  -h, --help                     Print help (see more with '--help')
  -V, --version                  Print version
```

## Dependências
- openssl-devel ou libssl
- pkg-config

Obs.: Algumas distros linux integram o openssl-devel junto com o pacote
openssl.

## Build
As seguintes instruções assumem que você tenha o ambiente de desenvolvimento
em Rust corretamente configurado. Qualquer dúvida recorra ao
[site oficial](https://www.rust-lang.org/).

Instale as dependências seguindo o
[rust-openssl setup](https://docs.rs/openssl/latest/openssl/#automatic)
e compile o projeto com:
```sh
cargo build --release
```

Para mais informações leia a sessão
["Building and Running a Cargo Project"](https://doc.rust-lang.org/stable/book/ch01-03-hello-cargo.html#building-and-running-a-cargo-project) do livro _The Rust Programming Language_.

## Entradas
A aplicação espera receber as seguintes entradas:

| Entrada    | Tipo    | Descrição    |
|---------------- | --------------- | --------------- |
| `event`    | Arquivo CSV    | Contém os dados do evento |
| `attendees`    | Arquivo CSV | Os participantes do evento |
| `cert-img` <sup>1</sup> | Texto | Imagem de um certificado que está no servidor (sem o caminho pro arquivo) |
| `upload-img` <sup>1</sup> | Imagem PNG 1122×792 | Imagem para subir pro servidor do site |
| `output` | Caminho para um arquivo (existente ou não) | Onde a saída do programa será gravada |

```
1. cert-img e upload-img não podem ser passados ao mesmo tempo. cert-img é
utilizada quando a imagem do certificado já é existente, já upload-img é 
usada quando uma nova imagem de certificado será utilizada.
```

O arquivo de `event` segue o seguinte template:

| NOME | DATA | TEXTO |
| --- | --- | --- |
| nome do evento | dia do evento no formato dia/mês/ano ou período do evento no formato dia/mês/ano - dia/mês/ano | id de um texto já existente (que pode ser encontrado no banco de dados na tabela 'texto') ou um novo texto de certificado (Ex.: Certificamos que #nome# de CPF/identificacao #identificacao# participou do evento #evento# com carga horaria de #cargaHoraria# hora(s) no #data#.) |

O arquivo de `attendees` segue o seguinte template:

| Nome    | CPF    | CH    |
|---------------- | --------------- | --------------- |
| nome do participante | CPF do participante do formato "000.000.000-00" | quantidade de horas cumpridas |

Para subir uma imagem para o servidor SFTP com é necessário que as variáveis
de ambiente `SFTP_ADDRESS`, `SFTP_USER` e `SFTP_PWD` estejam definidas. Para isso,
é recomendado a utilização de um arquivo `.env` que se parece com isso:

```env
# Isso é um comentário
SFTP_ADDRESS=algum.servidor:porta
SFTP_USER=usuario
SFTP_PWD='senha123'
```

O arquivo `.env` deve estar no diretório atual ou em algum de seus parentes.

## Saída
A aplicação escreve um arquivo .sql que deve ser importado no banco de dados
para a conclusão do cadastro dos certificados.
