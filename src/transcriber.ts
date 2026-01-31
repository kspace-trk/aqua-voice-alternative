import { GoogleGenerativeAI } from '@google/generative-ai';

const MODEL_NAME = 'gemini-3-flash-preview';
const TRANSCRIPTION_PROMPT = '音声を文字起こししてください。音声の内容のみを出力し、余計な説明は不要です。';

export async function transcribeAudio(
  apiKey: string,
  audioBase64: string
): Promise<string> {
  const genAI = new GoogleGenerativeAI(apiKey);
  console.log('Using API Key:', apiKey.substring(0, 5) + '...');
  const model = genAI.getGenerativeModel({ model: MODEL_NAME });

  const result = await model.generateContent([
    {
      inlineData: {
        mimeType: 'audio/webm',
        data: audioBase64,
      },
    },
    { text: TRANSCRIPTION_PROMPT },
  ]);

  const response = result.response;
  return response.text().trim();
}
